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
    static ref THREAD_URL_PATTERNS: Vec<Regex> = [r"/t/(?P<tid>\d+)(\?p=(?P<page>\d+))?",]
        .iter()
        .map(|re| Regex::new(re).unwrap())
        .collect();
    static ref TOTAL_PAGE_PATTERN: Regex = Regex::new(r"max=([0-9]+)").unwrap();
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct V2exForumConfig {
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

impl Default for V2exForumConfig {
    fn default() -> Self {
        Self {
            name: "V2ex".into(),
            base_url: "https://v2ex.com".into(),
            auth_method: AuthMethod::Guest,
            remove_ads: true,
            remove_user_info: true,
            remove_reply_box: true,
            interval_ms: default_interval(),
        }
    }
}

impl V2exForumConfig {
    pub async fn build_forum(&self) -> Result<V2exForum> {
        V2exForum::from_config(self.clone()).await
    }

    pub fn domain(&self) -> String {
        Url::parse(&self.base_url)
            .map(|url| url.host_str().unwrap_or_default().to_string())
            .unwrap_or_default()
    }
}

#[derive(Debug, Clone)]
pub struct V2exForum {
    pub config: V2exForumConfig,
    pub domain: String,
    pub client: Arc<Client>,
}

impl V2exForum {
    pub async fn try_new(name: &str, base_url_str: &str, auth_method: AuthMethod) -> Result<Self> {
        let config = V2exForumConfig {
            name: name.into(),
            base_url: base_url_str.trim_end_matches('/').to_string(),
            auth_method,
            ..Default::default()
        };
        Self::from_config(config).await
    }

    pub async fn from_config(config: V2exForumConfig) -> Result<Self> {
        let domain = config.domain();
        let client = config.auth_method.generate_client(&domain).await?;

        // 用户名密码登录
        if let AuthMethod::UsernamePassword {
            username: _,
            password: _,
        } = config.auth_method
        {
            return Err(Error::Login(format!(
                "{}不支持用户名密码登录，请使用其他登录方式",
                config.name
            )));
        }

        Ok(V2exForum {
            domain,
            config,
            client: Arc::new(client),
        })
    }
}

#[async_trait]
impl ForumProvider for V2exForum {
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
                let pn = captures
                    .name("page")
                    .and_then(|m| m.as_str().parse::<usize>().ok())
                    .unwrap_or(1);
                return Ok((thread_id, pn));
            }
        }
        return Err(Error::ThreadInfo(format!(
            "'{page_url}'不是标准的{}帖子网址",
            self.config.name
        )));
    }

    fn generate_thread_url(&self, tid: &str, pn: &str) -> String {
        if pn == "1" {
            format!("{}/t/{tid}", self.config.base_url)
        } else {
            format!("{}/t/{tid}?p={pn}", self.config.base_url)
        }
    }

    fn generate_filename(&self, tid: &str, pn: &str) -> String {
        format!("t-{tid}-{pn}.html")
    }

    fn extract_thread_info(&self, thread_url: &str, document: &NodeRef) -> Result<ThreadInfo> {
        let (thread_id, _) = self.extract_tid_pn(&thread_url)?;
        let title = extract_title(document)?;

        // 总页数，若获取失败则默认为1
        let total_pages = extract_total_pages(document).unwrap_or(1);
        let current_pn = extract_current_pn(document).unwrap_or(1);

        Ok(ThreadInfo {
            title,
            thread_id,
            total_pages,
            current_pn,
            page_size: 100, // V2EX默认每页100条回复
        })
    }

    async fn check_username(&self) -> Result<String> {
        let client = self.client.clone();
        let home_url = format!("{}/", self.config.base_url);
        let response = client.get(&home_url).send().await?.error_for_status()?;
        let text = response.text().await?;
        let document = parse_html().one(text);
        self.extract_username(&document)
            .ok_or(Error::Login("未登录".into()))
    }

    fn extract_username(&self, document: &NodeRef) -> Option<String> {
        document
            .select_first("a[href^='/member/']")
            .map(|node| node.text_contents().trim().to_string())
            .ok()
            .filter(|s| !s.is_empty())
    }

    fn preprocessing(&self, _thread_url: &str, document: NodeRef) -> Result<NodeRef> {
        if self.config.remove_user_info {
            // 删除顶部导航栏的用户信息
            if let Ok(node) = document.select_first("a[href^='/member/']") {
                node.as_node().detach();
            }
            // 删除Tweet链接中的用户名信息
            if let Ok(node) = document.select_first(r"a.tb[onclick^='window.open']") {
                node.as_node().detach();
            }
        }
        if let Ok(right_bar) = document.select_first("#Rightbar") {
            let right_bar = right_bar.as_node();
            if self.config.remove_user_info {
                // 删除右侧导航栏的用户信息
                if let Ok(node) = right_bar.select_first(".box") {
                    node.as_node().detach();
                }
            }
            if self.config.remove_ads {
                // 删除右侧导航栏广告
                if let Ok(node) = document.select_first("#pro-campaign-container") {
                    node.as_node().detach();
                }
            }
        }

        if self.config.remove_reply_box {
            // 删除回帖框
            if let Ok(node) = document.select_first("#reply-box") {
                node.as_node().detach();
            }
        }
        // 删除自动跳转script
        let head_ele = document
            .select_first("head")
            .map_err(|_| Error::HtmlParse("页面没有head元素".into()))?;
        if let Ok(script_eles) = head_ele.as_node().select("script") {
            for ele in script_eles {
                if ele.text_contents().contains("protectTraffic()") {
                    ele.as_node().detach();
                }
            }
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
                    let pn: usize = href.split("=").last().unwrap_or("1").parse().unwrap_or(1);
                    if pn > pn_last {
                        continue;
                    }
                    attrs_mut.insert("href", self.generate_filename(&thread_id, &pn.to_string()));
                }
            }

            // 替换输入跳转框
            if let Ok(input_ele) = pg_div.as_node().select_first("input.page_input") {
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

        // 处理分页元素
        let _ = document.select(".ps_container").map(|eles| {
            for pg_div in eles {
                let _ = process_ele_links(pg_div);
            }
        });

        // 方向键翻页
        let nav_node = generate_keyboard_nav_node(
            (pn - 1 >= pn_first)
                .then_some(self.generate_filename(&thread_id, &(pn - 1).to_string())),
            (pn + 1 <= pn_last)
                .then_some(self.generate_filename(&thread_id, &(pn + 1).to_string())),
        )?;
        document.append(nav_node);

        Ok(document)
    }

    fn interval_ms(&self) -> u64 {
        self.config.interval_ms
    }
}

// fn extract_thread_url(document: &NodeRef) -> Result<String> {
//     document
//         .select_first("meta[property='og:url']")
//         .map_err(|_| Error::ThreadInfo("获取url失败".into()))?
//         .attributes
//         .borrow()
//         .get("content")
//         .ok_or(Error::ThreadInfo("获取url失败".into()))
//         .map(|href| href.to_string())
// }

fn extract_title(document: &NodeRef) -> Result<String> {
    document
        .select_first("h1")
        .map_err(|_| Error::ThreadInfo("获取标题失败".into()))
        .map(|element| element.text_contents().replace("\n", "").trim().to_string())
}

fn extract_total_pages(document: &NodeRef) -> Result<usize> {
    if let Ok(input_ele) = document.select_first("input.page_input") {
        if let Some(max) = input_ele.attributes.borrow().get("max") {
            if let Ok(total) = max.parse::<usize>() {
                return Ok(total);
            }
        }
    }
    Ok(1)
}

fn extract_current_pn(document: &NodeRef) -> Result<usize> {
    if let Ok(input_ele) = document.select_first("input.page_input") {
        if let Some(value) = input_ele.attributes.borrow().get("value") {
            if let Ok(pn) = value.parse::<usize>() {
                return Ok(pn);
            }
        }
    }
    Ok(1)
}
