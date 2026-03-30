use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::Arc,
};

use crate::{core::ThreadInfo, error::Result, utils::clean_path};
use async_trait::async_trait;
use kuchiki::NodeRef;
use reqwest::Client;
use std::io::{BufWriter, Write};

#[async_trait]
pub trait ForumProvider: Send + Sync {
    /// 获取论坛名称
    fn name(&self) -> &str;
    /// 获取论坛域名
    fn domain(&self) -> &str;
    /// 获取论坛基础网址
    fn base_url(&self) -> &str;
    /// 验证URL是否属于此论坛
    fn match_url(&self, url: &str) -> bool;
    /// 获取帖子tid与页码
    fn extract_tid_pn(&self, page_url: &str) -> Result<(String, usize)>;
    /// 生成帖子地址
    fn generate_thread_url(&self, tid: &str, pn: &str) -> String;
    /// 帖子链接转为本地文件名
    fn generate_filename(&self, tid: &str, pn: &str) -> String;
    /// 从本地文件名中提取页码
    fn extract_pn_from_filename(&self, filename: &str, tid: &str) -> Option<usize>;
    /// 获取帖子信息
    fn extract_thread_info(&self, thread_url: &str, document: &NodeRef) -> Result<ThreadInfo>;
    /// 根据登陆设置生成客户端
    fn client(&self) -> Arc<Client>;
    /// 检查用户名
    async fn check_username(&self) -> Result<String>;
    fn extract_username(&self, document: &NodeRef) -> Option<String>;
    /// 前处理，包括删除广告、删除用户信息等等
    fn preprocessing(&self, thread_url: &str, document: NodeRef) -> Result<NodeRef>;
    /// 后处理，包括本地化翻页等
    fn postprocessing(&self, thread_url: &str, document: NodeRef) -> Result<NodeRef>;
    fn posts_sub_dir(&self) -> String {
        "posts".to_string()
    }
    fn assets_sub_dir(&self) -> String {
        "assets".to_string()
    }
    fn combine_pages(
        &self,
        base_dir: &Path,
        thread_info: &ThreadInfo,
        _page_map: HashMap<usize, PathBuf>,
    ) -> Result<PathBuf> {
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
    /// 帖子页面下载间隔，单位毫秒
    fn interval_ms(&self) -> u64;
}
