use std::sync::Arc;

use crate::core::{AuthMethod, ForumProvider, ThreadInfo};
use crate::error::{Error, Result};
use crate::utils::generate_keyboard_nav_node;
use async_trait::async_trait;
use kuchiki::traits::TendrilSink;
use kuchiki::{ElementData, NodeDataRef, NodeRef, parse_html};
use lazy_static::lazy_static;
use regex::Regex;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use url::Url;

lazy_static! {
    static ref THREAD_URL_PATTERNS: Vec<Regex> = [
        r"thread-(?P<tid>\d+)-(?P<page>\d+)",
        r"tid=(?P<tid>\d+)(?:.*&page=(?P<page>\d+))?",
    ]
    .iter()
    .map(|re| Regex::new(re).unwrap())
    .collect();
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscuzForumConfig {
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
    #[serde(default = "default_thread_url_template")]
    pub thread_url_template: String,
    #[serde(default)]
    pub selectors: DiscuzSelectors,
}
fn default_true() -> bool {
    true
}
fn default_interval() -> u64 {
    1500
}
fn default_thread_url_template() -> String {
    "thread-{tid}-{pn}-1.html".into()
}

impl Default for DiscuzForumConfig {
    fn default() -> Self {
        Self {
            name: "Discuz".into(),
            base_url: "".into(),
            auth_method: AuthMethod::Guest,
            remove_ads: true,
            remove_user_info: true,
            remove_reply_box: true,
            interval_ms: default_interval(),
            thread_url_template: default_thread_url_template(),
            selectors: DiscuzSelectors::default(),
        }
    }
}

impl DiscuzForumConfig {
    pub async fn build_forum(&self) -> Result<DiscuzForum> {
        DiscuzForum::from_config(self.clone()).await
    }
    pub fn domain(&self) -> String {
        Url::parse(&self.base_url)
            .map(|url| url.host_str().unwrap_or_default().to_string())
            .unwrap_or_default()
    }
}

#[derive(Debug, Clone)]
pub struct DiscuzForum {
    // pub name: String,
    // pub base_url: String,
    // pub domain: String,
    // pub auth_method: AuthMethod,
    pub config: DiscuzForumConfig,
    pub domain: String,
    pub client: Arc<Client>,
}

impl DiscuzForum {
    pub async fn try_new(name: &str, base_url_str: &str, auth_method: AuthMethod) -> Result<Self> {
        let config = DiscuzForumConfig {
            name: name.into(),
            base_url: base_url_str.trim_end_matches('/').to_string(),
            auth_method,
            ..Default::default()
        };
        Self::from_config(config).await
    }
    pub async fn from_config(config: DiscuzForumConfig) -> Result<Self> {
        let base_url_str = &config.base_url.trim_end_matches('/').to_string();
        let domain = config.domain();
        let client = config.auth_method.generate_client(&domain).await?;
        // 用户名密码登录
        if let AuthMethod::UsernamePassword {
            ref username,
            ref password,
        } = config.auth_method
        {
            let login_url = format!(
                "{base_url_str}/member.php?mod=logging&action=login&loginsubmit=yes&infloat=yes&lssubmit=yes&inajax=1"
            );
            let params = [("username", username), ("password", password)];
            let response = client
                .post(&login_url)
                .form(&params)
                .send()
                .await?
                .error_for_status()?;
            if response.text().await?.contains("登录失败") {
                return Err(Error::Login("登录失败，请检查用户名和密码".into()));
            }
        }
        Ok(DiscuzForum {
            domain,
            config,
            client: Arc::new(client),
        })
    }
}

#[async_trait]
impl ForumProvider for DiscuzForum {
    fn name(&self) -> &str {
        &self.config.name
    }
    fn domain(&self) -> &str {
        &self.domain
    }
    fn base_url(&self) -> &str {
        &self.config.base_url
    }
    fn client(&self) -> Arc<Client> {
        self.client.clone()
    }
    fn match_url(&self, url: &str) -> bool {
        url.starts_with(&self.config.base_url)
    }
    fn extract_tid_pn(&self, page_url: &str) -> Result<(String, usize)> {
        for pattern in THREAD_URL_PATTERNS.iter() {
            if let Some(captures) = pattern.captures(page_url) {
                let thread_id = captures["tid"].to_string();
                let pn = captures.name("page").map_or("1", |m| m.as_str());
                let pn = pn.parse::<usize>().map_err(|e| {
                    Error::ThreadInfo(format!(
                        "'{page_url}'不是标准的{}帖子网址: {e}",
                        self.config.name
                    ))
                })?;
                return Ok((thread_id, pn));
            }
        }
        return Err(Error::ThreadInfo(format!(
            "'{page_url}'不是标准的{}帖子网址",
            self.config.name
        )));
    }
    fn generate_thread_url(&self, tid: &str, pn: &str) -> String {
        // format!("{}/thread-{tid}-{pn}-1.html", self.config.base_url)
        format!(
            "{}/{}",
            self.config.base_url,
            self.config
                .thread_url_template
                .replace("{tid}", tid)
                .replace("{pn}", pn)
        )
    }
    fn generate_filename(&self, tid: &str, pn: &str) -> String {
        format!("thread-{tid}-{pn}-1.html")
    }
    fn extract_thread_info(&self, thread_url: &str, document: &NodeRef) -> Result<ThreadInfo> {
        let (thread_id, current_pn) = self.extract_tid_pn(&thread_url)?;
        let title = self.extract_title(document)?;

        // 总页数，若获取失败则默认为1
        let total_pages = self.extract_total_pages(document).unwrap_or(1);

        Ok(ThreadInfo {
            title,
            thread_id,
            total_pages,
            current_pn,
            page_size: 40,
        })
    }

    async fn check_username(&self) -> Result<String> {
        let client = self.client.clone();
        let home_url = format!(
            "{}/home.php?mod=space&do=thread&view=me",
            self.config.base_url
        );
        // let home_url = self.base_url.join("home.php?mod=space&do=thread&view=me")?.to_string();
        let response = client.get(&home_url).send().await?.error_for_status()?;
        let text = response.text().await?;
        let document = parse_html().one(text);
        self.extract_username(&document)
            .ok_or(Error::Login("未登录".into()))
    }
    fn extract_username(&self, document: &NodeRef) -> Option<String> {
        document
            .select_first(&self.config.selectors.username)
            .map(|node| node.text_contents().trim().to_string())
            .ok()
    }
    fn preprocessing(&self, _thread_url: &str, document: NodeRef) -> Result<NodeRef> {
        if self.config.remove_user_info {
            // 删除用户信息
            if let Ok(node) = document.select_first(&self.config.selectors.user_info) {
                node.as_node().detach();
            }
            // 未登录用户登录框
            else if let Ok(node) = document.select_first(&self.config.selectors.login_box) {
                node.as_node().detach();
            }
        }
        if self.config.remove_reply_box {
            // 删除回帖框
            if let Ok(node) = document.select_first(&self.config.selectors.reply_box) {
                node.as_node().detach();
            }
        }
        if self.config.remove_ads {
            // 删除广告
            if let Ok(eles) = document.select(&self.config.selectors.ads) {
                for node in eles {
                    node.as_node().detach();
                }
            }
        }
        // 强制utf-8编码
        if let Ok(meta_ele) = document.select_first(&self.config.selectors.charset) {
            meta_ele.as_node().detach();
        }
        Ok(document)
    }

    fn postprocessing(&self, thread_url: &str, document: NodeRef) -> Result<NodeRef> {
        let thread_info = self.extract_thread_info(thread_url, &document)?;
        let thread_id = thread_info.thread_id;
        let pn = thread_info.current_pn;
        let (pn_first, pn_last) = (1 as usize, thread_info.total_pages);
        let process_ele_links = |pg_div: NodeDataRef<ElementData>| -> Result<()> {
            let a_eles = pg_div
                .as_node()
                .select("a")
                .map_err(|_| Error::HtmlParse("未找到页码".to_string()))?;
            for a_ele in a_eles {
                let mut attrs_mut = a_ele.attributes.borrow_mut();
                if let Some(href) = attrs_mut.get("href") {
                    let Ok((tid, pn)) = self.extract_tid_pn(href) else {
                        continue;
                    };
                    if tid != thread_id || pn > pn_last {
                        continue;
                    }
                    attrs_mut.insert("href", self.generate_filename(&tid, &pn.to_string()));
                }
            }
            // 替换输入跳转框
            if let Ok(input_ele) = pg_div
                .as_node()
                .select_first(&self.config.selectors.pn_input)
            {
                let mut attrs_mut = input_ele.attributes.borrow_mut();
                attrs_mut.insert(
                    "onkeydown",
                    format!(
                        concat!(
                            "if(event.keyCode==13){{",
                            "window.location=`{}`;",
                            "doane(event);}}"
                        ),
                        self.generate_filename(&thread_id, "${this.value}")
                    ),
                );
            }
            Ok(())
        };

        // 2. 寻找所有 <div class="pg"> 元素
        let pg_divs = document
            .select(&self.config.selectors.pg_divs)
            .map_err(|_| Error::HtmlParse("未找到页码".to_string()))?;
        for pg_div in pg_divs {
            process_ele_links(pg_div)?;
        }
        // 下一页按钮，不一定存在
        if let Ok(pg_div) = document.select_first(&self.config.selectors.pgbtn) {
            process_ele_links(pg_div)?;
        }
        // 方向键翻页
        let page_nav_node = generate_keyboard_nav_node(
            (pn - 1 >= pn_first)
                .then_some(self.generate_filename(&thread_id, &(pn - 1).to_string())),
            (pn + 1 <= pn_last)
                .then_some(self.generate_filename(&thread_id, &(pn + 1).to_string())),
        )?;
        // 删除 document.onkeyup 事件
        if let Ok(script_eles) = document.select("script[type='text/javascript']") {
            for script_ele in script_eles {
                let script_text = script_ele.text_contents();
                if script_text.trim().starts_with("document.onkeyup") {
                    script_ele.as_node().detach();
                }
            }
        }
        document.append(page_nav_node);
        Ok(document)
    }
    fn interval_ms(&self) -> u64 {
        self.config.interval_ms
    }
}

impl DiscuzForum {
    fn extract_title(&self, document: &NodeRef) -> Result<String> {
        // document
        //     .select_first("h1.ts")
        //     .map_err(|_| Error::ThreadInfo("获取标题失败".into()))
        //     .map(|element| element.text_contents().replace("\n", "").trim().to_string())
        document
            .select_first(&self.config.selectors.thread_title)
            .map_err(|_| Error::ThreadInfo("获取标题失败".into()))
            .map(|element| element.text_contents().replace("\n", "").trim().to_string())
    }

    fn extract_total_pages(&self, document: &NodeRef) -> Result<usize> {
        let mut total_pages: usize = 1;
        if let Ok(pg_div) = document.select_first(&self.config.selectors.pg_divs) {
            let a_eles = pg_div
                .as_node()
                .select("a")
                .map_err(|_| Error::HtmlParse("未找到页码".to_string()))?;
            for a_ele in a_eles {
                let attrs = a_ele.attributes.borrow();
                if let Some(href) = attrs.get("href") {
                    if let Ok((_, pn)) = self.extract_tid_pn(href) {
                        total_pages = pn.max(total_pages);
                    };
                }
            }
        }
        Ok(total_pages)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct DiscuzSelectors {
    pub thread_title: String,
    pub pg_divs: String,
    pub pgbtn: String,
    pub pn_input: String,
    pub username: String,
    pub user_info: String,
    pub login_box: String,
    pub reply_box: String,
    pub ads: String,
    pub charset: String,
}
impl Default for DiscuzSelectors {
    fn default() -> Self {
        Self {
            thread_title: "h1".to_string(),
            pg_divs: "div.pg".to_string(),
            pgbtn: "div.pgbtn".to_string(),
            pn_input: "input.px[name='custompage']".to_string(),
            username: "strong.vwmy".to_string(),
            user_info: "#um".to_string(),
            login_box: "div.y.pns".to_string(),
            reply_box: "#f_pst".to_string(),
            ads: ".wp.a_h".to_string(),
            charset: "meta[charset='gbk']".to_string(),
        }
    }
}
