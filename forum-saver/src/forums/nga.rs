use std::collections::HashMap;
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::core::{AuthMethod, ForumProvider, ThreadInfo};
use crate::error::{Error, Result};
use crate::utils::{clean_path, create_node};
use async_trait::async_trait;
use kuchiki::NodeRef;
use lazy_static::lazy_static;
use regex::Regex;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use url::Url;

lazy_static! {
    static ref THREAD_URL_PATTERNS: Vec<Regex> = [r"tid=(?P<tid>\d+)(?:.*&page=(?P<page>\d+))?",]
        .iter()
        .map(|re| Regex::new(re).unwrap())
        .collect();
    static ref UNAME_PATTERN: Regex =
        Regex::new(r"__CURRENT_UNAME\s*=\s*'(?P<uname>.+?)',").unwrap();
    static ref UID_PATTERN: Regex = Regex::new(r#"__CURRENT_UID\s*=\s*(.+?),\n"#).unwrap();
    static ref TID_PATTERN: Regex = Regex::new(r"__CURRENT_TID=(?P<tid>\d+)").unwrap();
    static ref PN_PATTERN: Regex = Regex::new(r"__CURRENT_PAGE=(?P<pn>\d+)").unwrap();
    static ref __PAGE_PATTERN: Regex =
        Regex::new(r"var __PAGE = \{0:'/read.php\?tid=(?P<tid>\d+)',1:(?P<total_pages>\d+),2:(?P<pn>\d+),3:(?P<ps>\d+)\};").unwrap();
    // head script中定义的常量
    static ref VAR_PATTERN: Regex = Regex::new(r#"(__[A-Z0-9_]+)\s*=\s*['\"]([^'\";,]+)['\"]"#).unwrap();
    // 拼接URL
    static ref COMBINE_URL_PATTERN: Regex = Regex::new(r#"(__[A-Z0-9_]+)\+['\"](.+?)['\"]"#).unwrap();
    static ref IMG_URL_PATTERN: Regex = Regex::new(r#"\[img\](?P<img_url>[^\n]+?)\[/img\]"#).unwrap();
    static ref ADS_PATTERN: Regex =  Regex::new(r#","dsid_bbs_ads\d+":__DSBASE+.+?\n"#).unwrap();
    static ref SMILES_PATTERN: Regex = Regex::new(r#"\[s:(?P<group>[a-zA-Z0-9]+):(?P<name>.+?)\]"#).unwrap();
    static ref SMILES_MAP: HashMap<String, HashMap<String, String>> = {
        let json_str = include_str!("../../resources/js/nga_smiles.json");
        serde_json::from_str(json_str).unwrap()
    };
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NGAForumConfig {
    pub name: String,
    pub base_url: String,
    #[serde(default)]
    pub auth_method: AuthMethod,
    #[serde(default = "default_true")]
    pub remove_ads: bool,
    #[serde(default = "default_true")]
    pub remove_user_info: bool,
    #[serde(default = "default_true")]
    pub remove_reply_box: bool,
    #[serde(default = "default_interval")]
    pub interval_ms: u64,
}

fn default_true() -> bool {
    true
}

fn default_interval() -> u64 {
    1500
}
impl Default for NGAForumConfig {
    fn default() -> Self {
        Self {
            name: "NGA".into(),
            base_url: "https://nga.178.com".into(),
            auth_method: AuthMethod::default(),
            remove_ads: default_true(),
            remove_user_info: default_true(),
            remove_reply_box: default_true(),
            interval_ms: default_interval(),
        }
    }
}
impl NGAForumConfig {
    pub async fn build_forum(&self) -> Result<NGAForum> {
        NGAForum::from_config(self.clone()).await
    }

    pub fn domain(&self) -> String {
        Url::parse(&self.base_url)
            .map(|url| url.host_str().unwrap_or_default().to_string())
            .unwrap_or_default()
    }

    pub fn with_auth_method(self, auth_method: AuthMethod) -> Self {
        Self {
            auth_method,
            ..self
        }
    }
}
#[derive(Debug, Clone)]
pub struct NGAForum {
    pub config: NGAForumConfig,
    pub domain: String,
    pub client: Arc<Client>,
}

impl NGAForum {
    pub async fn from_config(config: NGAForumConfig) -> Result<Self> {
        let domain = config.domain();
        let client = config.auth_method.generate_client(&domain).await?;
        // 用户名密码登录
        if let AuthMethod::UsernamePassword {
            username: _,
            password: _,
        } = config.auth_method
        {
            return Err(Error::Login(format!(
                "{}用户名密码登录需要验证码，请使用Cookie登录",
                config.name
            )));
        } else if let AuthMethod::Guest = config.auth_method {
            return Err(Error::Login(format!(
                "{}限制游客访问，请使用Cookie登录后下载",
                config.name
            )));
        }
        Ok(Self {
            domain,
            config,
            client: Arc::new(client),
        })
    }
}

#[async_trait]
impl ForumProvider for NGAForum {
    fn name(&self) -> &str {
        &self.config.name
    }
    fn domain(&self) -> &str {
        &self.domain
    }
    fn client(&self) -> Arc<Client> {
        self.client.clone()
    }
    fn base_url(&self) -> &str {
        &self.config.base_url
    }
    fn match_url(&self, url: &str) -> bool {
        url.starts_with(&self.config.base_url)
    }
    fn extract_tid_pn(&self, page_url: &str) -> Result<(String, usize)> {
        for pattern in THREAD_URL_PATTERNS.iter() {
            if let Some(captures) = pattern.captures(page_url) {
                let thread_id = captures["tid"].to_string();
                let pn = captures
                    .get(2)
                    .map_or(Ok(1), |s| s.as_str().parse::<usize>())
                    .map_err(|e| {
                        Error::ThreadInfo(format!("'{page_url}'不是标准的NGA帖子网址: {e}"))
                    })?;
                return Ok((thread_id, pn));
            }
        }
        return Err(Error::ThreadInfo(format!(
            "'{page_url}'不是标准的NGA帖子网址"
        )));
    }
    fn generate_thread_url(&self, tid: &str, pn: &str) -> String {
        format!("{}/read.php?tid={tid}&page={pn}", self.config.base_url)
    }
    fn generate_filename(&self, tid: &str, pn: &str) -> String {
        format!("tid={tid}&page={pn}.html")
    }
    fn extract_pn_from_filename(&self, filename: &str, tid: &str) -> Option<usize> {
        let prefix = format!("tid={tid}&page=");
        let suffix = ".html";
        if filename.starts_with(&prefix) && filename.ends_with(suffix) {
            let pn_str = &filename[prefix.len()..filename.len() - suffix.len()];
            pn_str.parse::<usize>().ok()
        } else {
            None
        }
    }
    fn extract_thread_info(&self, thread_url: &str, document: &NodeRef) -> Result<ThreadInfo> {
        let (thread_id, pn) = self.extract_tid_pn(thread_url)?;
        let (_, total_pages, _, ps) =
            extract_pagination_info(document).unwrap_or(("".into(), 1, 1, 20));
        let title = document
            .select_first("h3#postsubject0")
            .map_err(|_| Error::ThreadInfo("获取标题失败".into()))?
            .text_contents();

        Ok(ThreadInfo {
            title,
            thread_id,
            total_pages,
            current_pn: pn,
            page_size: ps,
        })
    }

    async fn check_username(&self) -> Result<String> {
        let client = self.client.clone();
        let home_url = format!("https://{}", self.config.base_url);
        let response = client.get(&home_url).send().await?.error_for_status()?;
        let text = response.text().await?;
        let uname = UNAME_PATTERN
            .captures(&text)
            .map(|c| c[1].to_string())
            .ok_or_else(|| Error::Login("未登录".into()))?;
        Ok(uname)
    }
    fn extract_username(&self, document: &NodeRef) -> Option<String> {
        let text = document.text_contents();
        let uname = UNAME_PATTERN.captures(&text).map(|c| c[1].to_string());
        uname
    }
    fn preprocessing(&self, _thread_url: &str, document: NodeRef) -> Result<NodeRef> {
        // 强制utf-8编码
        if let Ok(meta_ele) = document.select_first("meta[http-equiv='Content-Type']") {
            meta_ele.as_node().detach();
        }

        // 解析__常量
        let mut config_vars = HashMap::new();

        if let Ok(script_ele) = document.select_first("script") {
            let script_ele_node = script_ele.as_node();
            let mut script_text = script_ele.text_contents();

            // 提取带引号的变量
            script_text = VAR_PATTERN
                .replace_all(&script_text, |cap: &regex::Captures| {
                    let (var_name, var_val) = (&cap[1], &cap[2]);
                    config_vars.insert(var_name.to_string(), var_val.to_string());
                    if var_val.starts_with("http") {
                        format!(
                            "{var_name}='../assets/{}'",
                            var_val.replace("http://", "").replace("https://", "")
                        )
                    } else {
                        cap[0].to_string()
                    }
                })
                .to_string();
            if let Some(img_style) = config_vars.get("__IMG_STYLE") {
                config_vars.insert("__RES_STYLE".to_string(), img_style.clone());
            }
            // 临时创建节点，插入需下载的文件路径
            script_text = COMBINE_URL_PATTERN
                .replace_all(&script_text, |cap: &regex::Captures| {
                    let (var_name, var_val) = (&cap[1], &cap[2]);
                    if let Some(value) = config_vars.get(&cap[1]) {
                        if let Ok(node) = create_node(
                            &format!(r#"<script src="{}{}" class="ngaPatch"/>"#, value, &cap[2]),
                            "script",
                        ) {
                            script_ele_node.insert_after(node)
                        }
                    }
                    format!(
                        r#"{var_name}+"{}""#,
                        if let Some((url, _)) = var_val.rsplit_once('?') {
                            url
                        } else {
                            var_val
                        }
                    )
                })
                .to_string();
            // 处理表情
            let img_path = config_vars
                .get("__IMG_PATH")
                .map(|s| s.as_str())
                .unwrap_or("https://img4.nga.178.com/ngabbs");
            SMILES_PATTERN
                .captures_iter(&document.text_contents())
                .for_each(|cap| {
                    let group = cap[1].to_string();
                    let name = cap[2].to_string();
                    if let Some(groups) = SMILES_MAP.get(&group) {
                        if let Some(img_url) = groups.get(&name) {
                            if let Ok(node) = create_node(
                                &format!(
                                    r#"<script src="{}/post/smile/{}" class="ngaPatch"/>"#,
                                    img_path, &img_url
                                ),
                                "script",
                            ) {
                                script_ele_node.insert_after(node)
                            }
                        }
                    }
                });

            script_text = script_text.replace("+Math.floor(__NOW/300)", "");
            if self.config.remove_ads {
                script_text = ADS_PATTERN.replace_all(&script_text, "").to_string();
            }
            if self.config.remove_user_info {
                script_text = UID_PATTERN.replace_all(&script_text, "").to_string();
                script_text = UNAME_PATTERN.replace_all(&script_text, "").to_string();
            }

            script_ele_node.insert_after(create_node(
                &format!(r#"<script type="text/javascript">{script_text}</script>"#),
                "script",
            )?);
            script_ele_node.detach();
        };
        if let Some(attach_base_view) = config_vars.get("__ATTACH_BASE_VIEW") {
            for cap in IMG_URL_PATTERN.captures_iter(&document.text_contents()) {
                let img_url = &cap["img_url"];
                let img_url_full = if img_url.starts_with("./") {
                    format!("https://{}/attachments/{}", attach_base_view, &img_url[2..])
                } else {
                    img_url.to_string()
                };
                document.append(create_node(
                    &format!(r#"<img src="{img_url_full}" class="ngaPatch"/>"#),
                    "img",
                )?);
            }
        }
        Ok(document)
    }

    fn postprocessing(&self, _thread_url: &str, document: NodeRef) -> Result<NodeRef> {
        // 删除所有class="ngaPatch"标签
        if let Ok(script_eles) = document.select(".ngaPatch") {
            let nodes_to_delete: Vec<_> = script_eles.map(|node| node.as_node().clone()).collect();
            for node in nodes_to_delete {
                node.detach();
            }
        }
        // 在<head>最后添加<script src="../assets/nga_patch.js"/>
        if let Ok(head_ele) = document.select_first("head") {
            let script_ele = create_node(r#"<script src="../assets/nga_patch.js"/>"#, "script")?;
            head_ele.as_node().insert_after(script_ele);
        }

        // 方向键翻页
        document.append(create_node(
            r#"<script type="text/javascript">makeKeyboardNav(__PAGE);</script>"#,
            "script",
        )?);

        Ok(document)
    }
    fn combine_pages(
        &self,
        base_dir: &Path,
        thread_info: &ThreadInfo,
        _page_map: HashMap<usize, PathBuf>,
    ) -> Result<PathBuf> {
        // 检查nga_patch.js是否存在
        let patch_js = base_dir.join(self.assets_sub_dir()).join("nga_patch.js");
        if !patch_js.exists() {
            write_patch_js(&patch_js)?;
        }
        let thread_id = &thread_info.thread_id;
        let thread_path = base_dir.join(&format!(
            "{thread_id}-{}.html",
            clean_path(&thread_info.title.replace("/", "_").replace("\\", "_"), "_")
        ));
        let mut writer = BufWriter::new(std::fs::File::create(&thread_path)?);
        writer.write_all(
            format!(
                r#"<meta http-equiv="refresh" content="0;url={}/{}">"#,
                self.posts_sub_dir(),
                self.generate_filename(&thread_id, "1")
            )
            .as_bytes(),
        )?;
        writer.flush()?;
        Ok(thread_path)
    }
    fn interval_ms(&self) -> u64 {
        self.config.interval_ms
    }
}

fn extract_pagination_info(document: &NodeRef) -> Result<(String, usize, usize, usize)> {
    if let Ok(eles) = document.select("script") {
        for script_ele in eles {
            let text = script_ele.text_contents();
            if !text.contains("__PAGE") {
                continue;
            }
            if let Some(caps) = __PAGE_PATTERN.captures(&text) {
                return Ok((
                    caps["tid"].to_string(),
                    caps["total_pages"].parse()?,
                    caps["pn"].parse()?,
                    caps["ps"].parse()?,
                ));
            }
        }
    }
    Err(Error::ThreadInfo("提取页码失败".into()))
}

fn write_patch_js(js_path: &Path) -> Result<()> {
    let embedded_js = include_str!("../../resources/js/nga_patch.js");
    let mut writer = BufWriter::new(std::fs::File::create(js_path)?);
    writer.write_all(embedded_js.as_bytes())?;
    writer.flush()?;
    Ok(())
}
