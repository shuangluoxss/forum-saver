use super::forum::ForumProvider;
use crate::core::types::DownloadInfo;
use crate::error::{Error, Result};
use crate::forums::ForumConfig;
use crate::i18n::I18n;
use crate::utils::{async_make_parent_dir, async_write_html, decode_response_text, url_to_path};
use dashmap::DashMap;
use futures_util::future::join_all;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use kuchiki::{ElementData, NodeDataRef, NodeRef, traits::TendrilSink};
use lazy_static::lazy_static;
use log::{error, info};
use pathdiff::diff_paths;
use regex::Regex;
use reqwest::{Client, Response};
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};
use tokio::{
    io::AsyncWriteExt,
    sync::{Semaphore, mpsc::Sender},
};
use url::Url;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum DownloadStrategy {
    FromStart,
    #[default]
    ResumeLatest,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct DownloaderConfig {
    /// Configuration for each forum provider.
    pub forums: Vec<ForumConfig>,
    /// HTML attributes that may contain downloadable resource links.
    pub downloadable_attrs: HashSet<String>,
    /// File extensions that are considered downloadable assets.
    pub downloadable_extensions: HashSet<String>,
    /// Maximum allowed length for a file path (to handle OS limitations).
    pub max_path_length: usize,
    /// Length of the hash used in generated asset filenames.
    pub path_hash_length: usize,
    /// Maximum recursion depth for asset discovery (e.g., CSS files referencing images).
    pub max_depth: usize,
    /// Root directory where all downloaded content will be stored.
    pub store_dir: PathBuf,
    /// Number of concurrent downloaders allowed.
    pub semaphore_count: usize,
    /// Language preference ("en" or "zh"), None means auto-detect from system
    pub language: Option<String>,
    /// Download strategy
    pub strategy: DownloadStrategy,
}

impl Default for DownloaderConfig {
    fn default() -> Self {
        Self {
            forums: Vec::new(),
            store_dir: "./data".into(),
            max_path_length: 240,
            path_hash_length: 16,
            max_depth: 3,
            semaphore_count: 8,
            strategy: DownloadStrategy::default(),
            downloadable_attrs: HashSet::from_iter(
                [
                    "href", "src", "data-src", "file", "zoomfile", "poster", "style",
                ]
                .iter()
                .map(|&s| s.into()),
            ),
            downloadable_extensions: HashSet::from_iter(
                [
                    "png", "jpg", "jpeg", "gif", "svg", "webp", "tiff", "bmp", "js", "mjs", "css",
                    "scss", "less", "woff", "woff2", "ttf", "eot", "otf", "mp4", "webm", "ogg",
                    "mp3", "wav", "aac", "flac",
                ]
                .iter()
                .map(|&s| s.into()),
            ),
            language: None,
        }
    }
}

impl DownloaderConfig {
    pub fn from_toml_file(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: Self = toml::from_str(&content)?;
        if config.forums.is_empty() {
            return Err(Error::Other("No forums found in the config file".into()));
        }
        Ok(config)
    }

    pub fn to_toml_file(&self, path: &Path) -> Result<()> {
        let content = toml::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }
}

lazy_static! {
    static ref CSS_URL_PATTERN: Regex = Regex::new(r#"url\((['"]?)(.+?)['"]?\)"#).unwrap();
    static ref INVALID_PATH_CHARACTERS: Regex = Regex::new(r#"[<>:"|?*]+"#).unwrap();
}

/// The core downloader responsible for fetching thread pages and their associated assets.
pub struct Downloader {
    /// Mapping of domain names to their respective forum providers.
    forum_map: Arc<DashMap<String, Arc<dyn ForumProvider>>>,
    /// Mapping of domain names to their respective forum configurations.
    forum_config_map: Arc<HashMap<String, ForumConfig>>,
    /// Semaphore to limit the number of concurrent network requests.
    semaphore: Arc<Semaphore>,
    /// Thread-safe map to track downloaded assets and avoid redundant downloads.
    asset_map: Arc<DashMap<String, Option<PathBuf>>>,
    /// Downloader configuration parameters.
    pub config: DownloaderConfig,
    /// Internationalization handler
    pub i18n: Arc<I18n>,
}

impl Downloader {
    pub fn from_config(config: DownloaderConfig) -> Result<Self> {
        let forum_config_map = config
            .forums
            .iter()
            .map(|forum_config| {
                let domain = forum_config.domain();

                (domain, forum_config.clone())
            })
            .collect::<HashMap<_, _>>();

        let i18n = I18n::new(config.language.as_deref());

        Ok(Self {
            forum_map: Arc::new(DashMap::new()),
            forum_config_map: Arc::new(forum_config_map),
            semaphore: Arc::new(Semaphore::new(config.semaphore_count)),
            asset_map: Arc::new(DashMap::new()),
            config,
            i18n: Arc::new(i18n),
        })
    }
    pub async fn get_forum(&self, domain: &str) -> Result<Arc<dyn ForumProvider>> {
        if let Some(forum) = self.forum_map.get(domain) {
            return Ok(forum.clone());
        } else if let Some(forum_config) = self.forum_config_map.get(domain) {
            let forum = forum_config.build_forum().await?;
            self.forum_map.insert(domain.to_string(), forum.clone());
            Ok(forum)
        } else {
            let available_forums: Vec<String> = self
                .forum_config_map
                .keys()
                .map(|domain| format!("'{domain}'"))
                .collect();
            Err(Error::InvalidForum(format!(
                "'{domain}' not in supported forums: [{}]",
                available_forums.join(", ")
            )))
        }
    }

    /// Downloads a forum thread by its URL, processing all pages and assets.
    pub async fn download_thread(
        &self,
        thread_url: &str,
        channel: Option<Sender<DownloadInfo>>,
    ) -> Result<()> {
        // 发送线程URL信息
        {
            let message = self
                .i18n
                .t("downloading-thread", Some(&[("url", thread_url)]));
            info!("{message}");
            if let Some(ref sender) = channel {
                let _ = sender.send(DownloadInfo::new_info(message)).await;
            }
        }
        let parsed_url = Url::parse(thread_url)?;
        let domain = parsed_url.domain().unwrap_or("");
        let forum = &self.get_forum(domain).await?;
        let client = forum.client();
        let (thread_id, _) = forum.extract_tid_pn(thread_url)?;
        let store_dir = self.config.store_dir.join(forum.name());
        let html_dir = store_dir.join(forum.posts_sub_dir());
        let first_page_path = html_dir.join(&forum.generate_filename(&thread_id, "1"));
        async_make_parent_dir(&first_page_path).await?;
        let asset_dir = store_dir.join(forum.assets_sub_dir());
        tokio::fs::create_dir_all(&asset_dir).await?;
        let mp = Arc::new(MultiProgress::new());
        // Fetch the first page without recursive asset adaptation to get thread info.
        let first_page_url = forum.generate_thread_url(&thread_id, "1");
        let first_page = self
            .fetch_html(
                client.clone(),
                &first_page_url,
                Some(mp.clone()),
                channel.clone(),
            )
            .await?;
        let thread_info = forum.extract_thread_info(thread_url, &first_page)?;
        match forum.extract_username(&first_page) {
            Some(username) => {
                // 发送用户名信息
                let message = self.i18n.t(
                    "username",
                    Some(&[
                        ("forum", forum.name()),
                        ("username", &format!("{:?}", username)),
                    ]),
                );
                if let Some(ref sender) = channel {
                    let _ = sender.send(DownloadInfo::new_info(message.clone())).await;
                }
                info!("{message}")
            }
            None => {
                // 发送未登录信息
                let message = self.i18n.t("not-login", Some(&[("forum", forum.name())]));
                info!("{message}");
                if let Some(ref sender) = channel {
                    let _ = sender.send(DownloadInfo::new_info(message)).await;
                }
            }
        };

        // 发送开始下载信息
        if let Some(ref sender) = channel {
            let message = self.i18n.t(
                "started-downloading",
                Some(&[
                    ("title", &thread_info.title),
                    ("pages", &thread_info.total_pages.to_string()),
                ]),
            );
            info!("{message}");
            let _ = sender.send(DownloadInfo::new_info(message)).await;
        }

        let pb_spinner = mp.add(ProgressBar::new_spinner());
        pb_spinner.set_style(ProgressStyle::with_template("{spinner:.green} {msg}").unwrap());
        pb_spinner.enable_steady_tick(Duration::from_millis(100));

        // 检查本地已有的页码
        let mut start_pn = 1;
        if matches!(self.config.strategy, DownloadStrategy::ResumeLatest) {
            if let Ok(mut entries) = tokio::fs::read_dir(&html_dir).await {
                while let Some(entry) = entries.next_entry().await.ok().flatten() {
                    let filename = entry.file_name().to_string_lossy().to_string();
                    if let Some(pn) = forum.extract_pn_from_filename(&filename, &thread_id) {
                        start_pn = start_pn.max(pn);
                    }
                }
            }
            if start_pn > 1 {
                let message = self.i18n.t("resume-from-page", Some(&[("page", &start_pn.to_string())]));
                info!("{message}");
                if let Some(ref sender) = channel {
                    let _ = sender.send(DownloadInfo::new_info(message)).await;
                }
            }
        }

        // 发送初始化消息
        let init_msg = self.i18n.t("initializing", None);
        pb_spinner.set_message(init_msg.clone());
        if let Some(ref sender) = channel {
            let _ = sender.send(DownloadInfo::new_info(init_msg)).await;
        }

        let total_download_pages = if start_pn > 1 {
            thread_info.total_pages.saturating_sub(start_pn) + 2 // page 1 + [start_pn..total_pages]
        } else {
            thread_info.total_pages
        };

        let pb_process = mp.add(ProgressBar::new(total_download_pages as u64));
        pb_process.set_style(
            ProgressStyle::with_template("{wide_bar:.cyan/blue} {pos:>3}/{len:3} {msg}").unwrap(),
        );
        pb_process.set_position(0);

        let args = DownloadArgs {
            client: client.clone(),
            url_str: forum.generate_thread_url(&thread_id, "1"),
            file_path: html_dir.join(forum.generate_filename(&thread_id, "1")),
            asset_dir: asset_dir,
            max_depth: self.config.max_depth,
            mp: Some(mp.clone()),
            channel: channel.clone(),
        };
        let fetch_tasks: Vec<_> = (start_pn.max(2)..=thread_info.total_pages)
            .map(|pn| {
                let thread_id = thread_id.clone();
                let html_url = forum.generate_thread_url(&thread_id, &pn.to_string());
                let file_path = html_dir.join(forum.generate_filename(&thread_id, &pn.to_string()));
                let args_clone = args
                    .clone()
                    .with_url_str(&html_url)
                    .with_file_path(&file_path);
                let pb_process = pb_process.clone();
                let channel = channel.clone();
                async move {
                    let res: Result<(usize, PathBuf)> = async {
                        let doc = self
                            .fetch_html(
                                args_clone.client.clone(),
                                &html_url,
                                args_clone.mp.clone(),
                                channel.clone(),
                            )
                            .await?;
                        let doc = forum.preprocessing(&html_url, doc)?;
                        let doc = self.adapt_html_document(doc, &args_clone).await?;
                        let doc = forum.postprocessing(&html_url, doc)?;
                        async_write_html(doc, &args_clone.file_path).await?;
                        Ok((pn, args_clone.file_path))
                    }
                    .await;
                    // Update progress bar regardless of task success.
                    pb_process.inc(1);

                    // 发送进度更新
                    if let Some(ref sender) = channel {
                        let _ = sender
                            .send(DownloadInfo::new_progress(
                                pb_process.position(),
                                pb_process.length().unwrap_or(0),
                            ))
                            .await;
                    }

                    res
                }
            })
            .collect();
        // Process the first page separately as it was fetched earlier.
        let res: Result<(usize, PathBuf)> = async {
            let doc = forum.preprocessing(&first_page_url, first_page)?;
            let doc = self.adapt_html_document(doc, &args).await?;
            let doc = forum.postprocessing(&first_page_url, doc)?;
            async_write_html(doc, &args.file_path).await?;
            Ok((1, args.file_path.clone()))
        }
        .await;
        pb_process.inc(1);

        // 发送进度更新
        if let Some(ref sender) = channel {
            let _ = sender
                .send(DownloadInfo::new_progress(
                    pb_process.position(),
                    pb_process.length().unwrap_or(0),
                ))
                .await;
        }

        let mut page_map = HashMap::new();
        match res {
            Ok((pn, file_path)) => page_map.insert(pn, file_path),
            Err(e) => {
                // 发送错误信息
                let message = self.i18n.t(
                    "error-fetching-page",
                    Some(&[("page", "1"), ("error", &format!("{:?}", e))]),
                );
                error!("{message}\n");
                if let Some(ref sender) = channel {
                    let _ = sender.send(DownloadInfo::new_error(message)).await;
                }

                None
            }
        };
        // Sequential fetching of remaining pages with anti-scraping intervals.
        let interval_ms = forum.interval_ms();
        if interval_ms > 0 {
            let duration = Duration::from_millis(interval_ms);
            let total_pages = thread_info.total_pages;
            let start_pn_loop = start_pn.max(2);
            for (i, task) in fetch_tasks.into_iter().enumerate() {
                let current_pn = i + start_pn_loop;
                let page_saved_msg = self.i18n.t(
                    "page-saved",
                    Some(&[
                        ("current", &(current_pn - 1).to_string()),
                        ("total", &total_pages.to_string()),
                        ("ms", &interval_ms.to_string()),
                    ]),
                );
                pb_spinner.set_message(page_saved_msg.clone());

                // 发送页面保存信息
                if let Some(ref sender) = channel {
                    let _ = sender.send(DownloadInfo::new_info(page_saved_msg)).await;
                }

                tokio::time::sleep(duration).await;
                let requesting_page_msg = self.i18n.t(
                    "requesting-page",
                    Some(&[
                        ("current", &current_pn.to_string()),
                        ("total", &total_pages.to_string()),
                    ]),
                );
                pb_spinner.set_message(requesting_page_msg.clone());

                // 发送请求页面信息
                if let Some(ref sender) = channel {
                    let _ = sender
                        .send(DownloadInfo::new_info(requesting_page_msg))
                        .await;
                }

                match task.await {
                    Ok((pn, file_path)) => page_map.insert(pn, file_path),
                    Err(e) => {
                        // 发送错误信息
                        let message = self.i18n.t(
                            "error-fetching-page",
                            Some(&[
                                ("page", &current_pn.to_string()),
                                ("error", &format!("{:?}", e)),
                            ]),
                        );
                        error!("{message}\n");
                        if let Some(ref sender) = channel {
                            let _ = sender.send(DownloadInfo::new_error(message)).await;
                        }

                        None
                    }
                };
            }
        } else {
            let interval_ms_zero_msg = self.i18n.t("interval-ms-zero", None);
            pb_spinner.set_message(interval_ms_zero_msg.clone());

            // 发送间隔为零信息
            if let Some(ref sender) = channel {
                let _ = sender
                    .send(DownloadInfo::new_info(interval_ms_zero_msg))
                    .await;
            }

            for res in join_all(fetch_tasks).await {
                match res {
                    Ok((pn, file_path)) => page_map.insert(pn, file_path),
                    Err(e) => {
                        // 发送错误信息
                        let message = self.i18n.t(
                            "error-fetching-page",
                            Some(&[("page", "unknown"), ("error", &format!("{:?}", e))]),
                        );
                        error!("{message}\n");
                        if let Some(ref sender) = channel {
                            let _ = sender.send(DownloadInfo::new_error(message)).await;
                        }

                        None
                    }
                };
            }
        }
        let all_pages_fetched_msg = self.i18n.t("all-pages-fetched", None);
        pb_spinner.set_message(all_pages_fetched_msg.clone());

        // 发送所有页面已获取信息
        if let Some(ref sender) = channel {
            let _ = sender
                .send(DownloadInfo::new_info(all_pages_fetched_msg))
                .await;
        }

        let thread_path = forum.combine_pages(&store_dir, &thread_info, page_map)?;
        mp.clear()?;

        // 发送线程完成信息
        {
            let message = self.i18n.t(
                "thread-complete",
                Some(&[("path", &thread_path.display().to_string())]),
            );
            info!("{message}");
            if let Some(ref sender) = channel {
                let _ = sender
                    .send(DownloadInfo::new_text(message, "success".to_string()))
                    .await;
                let _ = sender.send(DownloadInfo::new_progress(100, 100)).await;
            }
        }

        Ok(())
    }

    /// Fetches an HTML page and adapts it by downloading and localizing its assets.
    pub async fn fetch_and_adapt_html(&self, args: &DownloadArgs) -> Result<NodeRef> {
        let document = self
            .fetch_html(
                args.client.clone(),
                &args.url_str,
                args.mp.clone(),
                args.channel.clone(),
            )
            .await?;
        self.adapt_html_document(document, args).await
    }

    /// Fetches the raw HTML content from a URL and parses it into a DOM tree.
    pub async fn fetch_html(
        &self,
        client: Arc<Client>,
        html_url: &str,
        mp: Option<Arc<MultiProgress>>,
        channel: Option<Sender<DownloadInfo>>,
    ) -> Result<NodeRef> {
        let response = self.get_with_premit(client, html_url, mp, channel).await?;
        // Try to detect encoding from headers before consuming the response
        let text = decode_response_text(response).await?;

        Ok(kuchiki::parse_html().one(text))
    }

    /// Processes an HTML document to find, download, and localize its static assets.
    pub async fn adapt_html_document(
        &self,
        document: NodeRef,
        args: &DownloadArgs,
    ) -> Result<NodeRef> {
        let html_dir = args.file_path.parent().ok_or_else(|| {
            Error::Path(format!("Invalid output path: {}", args.file_path.display()))
        })?;
        let html_dir = std::fs::canonicalize(&html_dir)?;
        let asset_dir = std::fs::canonicalize(&args.asset_dir)?;
        // Stop recursion if max depth is reached.
        if args.max_depth <= 0 {
            return Ok(document);
        }
        let html_url = args.url_str.to_string();
        let base_url_str = match document.select_first("base") {
            Ok(ele) => {
                let s = ele
                    .attributes
                    .borrow()
                    .get("href")
                    .map(|s| s.to_string())
                    .unwrap_or(html_url);
                ele.as_node().detach();
                s
            }
            Err(_) => html_url,
        };
        let base_url = Url::parse(&base_url_str)?.join(".")?;
        // Vectors to store discovered assets and CSS attributes for later processing.
        let mut assets = Vec::new();
        let mut css_attrs = Vec::new();

        // Iterate through configured downloadable attributes (e.g., src, href).
        for attr_name in &self.config.downloadable_attrs {
            let Ok(eles) = document.select(&format!("*[{attr_name}]")) else {
                continue;
            };

            for ele in eles {
                let attr_value = {
                    let attrs = ele.attributes.borrow();
                    attrs.get(attr_name.as_str()).unwrap_or("").to_string()
                };
                if attr_value.is_empty() {
                    continue;
                }

                // Handle inline styles or attributes containing url() patterns.
                if CSS_URL_PATTERN.captures(&attr_value).is_some() {
                    let mut resources = HashMap::new();
                    for cap in CSS_URL_PATTERN.captures_iter(&attr_value) {
                        let raw_url = cap.get(2).map_or("", |m| m.as_str());
                        if raw_url.is_empty() || resources.contains_key(raw_url) {
                            continue;
                        }
                        let abs_url = match base_url.join(raw_url) {
                            Ok(url) => url,
                            Err(_) => continue,
                        };
                        if !abs_url.scheme().starts_with("http") {
                            continue;
                        }
                        let abs_url_str = abs_url.to_string();
                        let mut abs_path = PathBuf::new();
                        let mut rel_path_str = String::new();

                        let url_extension = abs_url.path().split('.').last().unwrap_or("");
                        if self.config.downloadable_extensions.contains(url_extension) {
                            let (path, rel) =
                                self.resolve_resource_paths(&abs_url, &asset_dir, &html_dir)?;
                            abs_path = path;
                            rel_path_str = rel;
                        }

                        resources
                            .insert(raw_url.to_string(), (abs_url_str, abs_path, rel_path_str));
                    }
                    if resources.is_empty() {
                        continue;
                    }
                    css_attrs.push(CssAttrItem {
                        node: ele.clone(),
                        attr_name: attr_name.clone(),
                        value: attr_value,
                        resources,
                    });
                    continue;
                }

                let abs_url = match base_url.join(&attr_value) {
                    Ok(url) => url,
                    Err(_) => continue,
                };
                if !abs_url.scheme().starts_with("http") {
                    continue;
                }

                let abs_url_str = abs_url.to_string();

                // Normalize attribute to absolute URL before downloading.
                {
                    let mut attrs_mut = ele.attributes.borrow_mut();
                    attrs_mut.insert(attr_name.clone(), abs_url_str.clone());
                }

                let url_extension = abs_url.path().split('.').last().unwrap_or("");
                if !self.config.downloadable_extensions.contains(url_extension) {
                    continue;
                }

                let (abs_path, rel_path_str) =
                    self.resolve_resource_paths(&abs_url, &asset_dir, &html_dir)?;

                assets.push(AssetItem::new(
                    ele.clone(),
                    attr_name.clone(),
                    abs_url_str,
                    abs_path,
                    rel_path_str,
                ));
            }
        }

        // Deduplicate URLs and prepare download tasks.
        let mut unique_url_path_map: HashMap<_, _> = assets
            .iter()
            .map(|item| (item.abs_url.clone(), item.abs_path.clone()))
            .collect();
        for css_attr in &css_attrs {
            for (_, (abs_url_str, abs_path, rel_path)) in css_attr.resources.iter() {
                if !abs_path.as_os_str().is_empty() && !rel_path.is_empty() {
                    unique_url_path_map
                        .entry(abs_url_str.clone())
                        .or_insert(abs_path.clone());
                }
            }
        }

        let tasks = unique_url_path_map.into_iter().map(|(abs_url, abs_path)| {
            let args_clone = args
                .clone()
                .with_url_str(&abs_url)
                .with_file_path(abs_path)
                .with_max_depth(args.max_depth - 1);
            async move {
                self.fetch_and_adapt_asset(&args_clone)
                    .await
                    .ok()
                    .map(|_| abs_url)
            }
        });

        // Collect URLs of successfully downloaded assets.
        let success_abs_urls: HashSet<String> = join_all(tasks)
            .await
            .into_iter()
            .filter_map(|abs_url| abs_url)
            .collect();

        // Update DOM attributes with local relative paths only for successful downloads.
        for item in assets {
            if success_abs_urls.contains(&item.abs_url) {
                item.node
                    .attributes
                    .borrow_mut()
                    .insert(item.attr_name, item.attr_value_new);
            }
        }

        for css_attr in css_attrs {
            let CssAttrItem {
                node,
                attr_name,
                value,
                resources,
            } = css_attr;
            let new_value = CSS_URL_PATTERN
                .replace_all(&value, |caps: &regex::Captures| {
                    let quote = &caps[1];
                    let raw_url = &caps[2];

                    if let Some((abs_url_str, _, rel_path)) = resources.get(raw_url) {
                        let target_url =
                            if !rel_path.is_empty() && success_abs_urls.contains(abs_url_str) {
                                rel_path
                            } else {
                                abs_url_str
                            };
                        return format!("url({quote}{target_url}{quote})");
                    }
                    caps[0].to_string()
                })
                .into_owned();

            node.attributes.borrow_mut().insert(attr_name, new_value);
        }
        Ok(document)
    }

    /// Fetches a CSS file and adapts it by downloading and localizing its internal resources.
    pub async fn fetch_and_adapt_css(&self, args: &DownloadArgs) -> Result<String> {
        let response = self
            .get_with_premit(
                args.client.clone(),
                &args.url_str,
                args.mp.clone(),
                args.channel.clone(),
            )
            .await?;

        let css_text = decode_response_text(response).await?;

        // Stop recursion if max depth is reached.
        if args.max_depth <= 0 {
            return Ok(css_text);
        }

        let out_dir = args.file_path.parent().ok_or_else(|| {
            Error::Path(format!("Invalid output path: {}", args.file_path.display()))
        })?;
        async_make_parent_dir(&args.file_path).await?;
        let css_dir = std::fs::canonicalize(out_dir)?;
        let asset_dir = std::fs::canonicalize(&args.asset_dir)?;
        let base_url = Url::parse(&args.url_str)?.join(".")?;
        // Map of raw URLs to their resolved absolute URLs and local paths.
        let mut resource_map = HashMap::new();

        // Parse all valid resource links from CSS.
        for cap in CSS_URL_PATTERN.captures_iter(&css_text) {
            let raw_url = cap.get(2).map_or("", |m| m.as_str());
            if raw_url.is_empty() || resource_map.contains_key(raw_url) {
                continue;
            }

            let abs_url = base_url.join(raw_url)?;
            if !abs_url.scheme().starts_with("http") {
                continue;
            }
            let (abs_path, rel_path_str) =
                self.resolve_resource_paths(&abs_url, &asset_dir, &css_dir)?;

            resource_map.insert(
                raw_url.to_string(),
                (abs_url.to_string(), abs_path, rel_path_str),
            );
        }

        // Concurrent download of discovered CSS resources.
        let tasks = resource_map.values().map(|(abs_url_str, abs_path, _)| {
            let args_clone = args
                .clone()
                .with_url_str(abs_url_str)
                .with_file_path(abs_path)
                .with_max_depth(args.max_depth - 1);
            async move {
                self.fetch_and_adapt_asset(&args_clone)
                    .await
                    .ok()
                    .map(|_| abs_url_str.to_string())
            }
        });

        let success_url_strs: HashSet<String> = join_all(tasks)
            .await
            .into_iter()
            .filter_map(|abs_url_str| abs_url_str)
            .collect();

        // Replace resource URLs in CSS text with local paths.
        let css_text_new = CSS_URL_PATTERN.replace_all(&css_text, |caps: &regex::Captures| {
            let full_match = &caps[0]; // e.g., url("...")
            let quote = &caps[1];
            let raw_url = &caps[2];

            if let Some((abs_url_str, _, rel_path)) = resource_map.get(raw_url) {
                let target_url = if success_url_strs.contains(abs_url_str) {
                    rel_path
                } else {
                    abs_url_str
                };
                return format!("url({quote}{target_url}{quote})");
            }
            full_match.to_string()
        });
        Ok(css_text_new.to_string())
    }

    /// Fetches an individual asset, handling HTML and CSS files recursively.
    pub async fn fetch_and_adapt_asset(&self, args: &DownloadArgs) -> Result<()> {
        if self.asset_map.contains_key(&args.url_str) {
            return Ok(());
        } else if args.file_path.exists() {
            self.asset_map
                .insert(args.url_str.clone(), Some(args.file_path.clone()));
            return Ok(());
        }
        let parsed_url = Url::parse(&args.url_str)?;
        let url_extensition = parsed_url.path().split(".").last().unwrap_or("");

        // Prevent duplicate concurrent downloads of the same asset.
        self.asset_map.insert(args.url_str.to_string(), None);
        match url_extensition {
            // HTML assets are processed recursively for their own dependencies.
            "htm" | "html" => {
                let document = self.fetch_and_adapt_html(args).await?;
                async_make_parent_dir(&args.file_path).await?;
                async_write_html(document, &args.file_path).await?;
            }
            // CSS assets are processed recursively for their own dependencies.
            "css" | "scss" | "less" => {
                let css_text = self.fetch_and_adapt_css(args).await?;
                async_make_parent_dir(&args.file_path).await?;
                let mut writer =
                    tokio::io::BufWriter::new(tokio::fs::File::create(&args.file_path).await?);
                writer.write_all(css_text.as_bytes()).await?;
                writer.flush().await?;
            }
            // JavaScript assets are downloaded as streams.
            "js" | "mjs" => {
                let response = self
                    .get_with_premit(
                        args.client.clone(),
                        &args.url_str,
                        args.mp.clone(),
                        args.channel.clone(),
                    )
                    .await?;
                let js_text = decode_response_text(response).await?;
                async_make_parent_dir(&args.file_path).await?;
                let mut writer =
                    tokio::io::BufWriter::new(tokio::fs::File::create(&args.file_path).await?);
                writer.write_all(js_text.as_bytes()).await?;
                writer.flush().await?;
            }
            // Binary or other static assets are downloaded as streams.
            _ => {
                let mut response = self
                    .get_with_premit(
                        args.client.clone(),
                        &args.url_str,
                        args.mp.clone(),
                        args.channel.clone(),
                    )
                    .await?;
                async_make_parent_dir(&args.file_path).await?;
                let mut writer =
                    tokio::io::BufWriter::new(tokio::fs::File::create(&args.file_path).await?);
                while let Some(chunk) = response.chunk().await? {
                    writer.write_all(&chunk).await?;
                }
                writer.flush().await?;
            }
        }
        // Mark asset as successfully downloaded in the shared map.
        self.asset_map
            .insert(args.url_str.to_string(), Some(args.file_path.clone()));
        Ok(())
    }
    /// Resolves absolute URLs to local file paths and their relative paths for localization.
    fn resolve_resource_paths(
        &self,
        abs_url: &Url,
        asset_dir: &Path,
        base_dir: &Path,
    ) -> Result<(PathBuf, String)> {
        let abs_path = url_to_path(
            abs_url,
            asset_dir,
            "index",
            self.config.max_path_length,
            self.config.path_hash_length,
        )?;
        let rel_path = diff_paths(&abs_path, base_dir)
            .ok_or_else(|| Error::Path(format!("Path error: {abs_path:?}")))?
            .to_string_lossy()
            .replace("\\", "/");
        Ok((abs_path, rel_path))
    }

    /// Performs a network GET request with concurrency control and optional progress reporting.
    async fn get_with_premit(
        &self,
        client: Arc<Client>,
        url: &str,
        mp: Option<Arc<MultiProgress>>,
        channel: Option<Sender<DownloadInfo>>,
    ) -> Result<Response> {
        let _premit = self.semaphore.acquire().await?;

        // 发送下载资产信息
        let message = self.i18n.t("downloading-asset", Some(&[("url", url)]));
        if let Some(ref sender) = channel {
            let _ = sender.send(DownloadInfo::new_info(message.clone())).await;
        }

        let response = match mp {
            Some(ref mp) => {
                let pb = mp.add(ProgressBar::new_spinner());
                pb.set_style(ProgressStyle::with_template("  {spinner:.green} {msg}").unwrap());
                pb.enable_steady_tick(Duration::from_millis(100));
                pb.set_message(message);
                let response = client.get(url).send().await;
                pb.finish_and_clear();
                mp.remove(&pb);
                response?
            }
            None => client.get(url).send().await?,
        };
        Ok(response.error_for_status()?)
    }

    pub fn supported_forums(&self) -> Vec<String> {
        self.forum_config_map.keys().cloned().collect()
    }
}

/// Represents an asset found within an HTML document.
struct AssetItem {
    /// The DOM element containing the asset reference.
    node: NodeDataRef<ElementData>,
    /// The name of the attribute (e.g., "src", "href") where the link was found.
    attr_name: String,
    /// The absolute URL of the asset.
    abs_url: String,
    /// The local absolute path where the asset will be stored.
    abs_path: PathBuf,
    /// The relative path used to update the DOM if download succeeds.
    attr_value_new: String,
}

impl AssetItem {
    fn new(
        node: NodeDataRef<ElementData>,
        attr_name: impl ToString,
        abs_url: impl ToString,
        abs_path: impl Into<PathBuf>,
        attr_value_new: impl ToString,
    ) -> Self {
        Self {
            node,
            attr_name: attr_name.to_string(),
            abs_url: abs_url.to_string(),
            abs_path: abs_path.into(),
            attr_value_new: attr_value_new.to_string(),
        }
    }
}

/// Represents a collection of resource references found within a single CSS attribute.
struct CssAttrItem {
    /// The DOM element containing the style attribute.
    node: NodeDataRef<ElementData>,
    /// The name of the attribute containing CSS (e.g., "style").
    attr_name: String,
    /// The original value of the attribute.
    value: String,
    /// Mapping of raw resource strings to their absolute URLs, local paths, and relative paths.
    resources: HashMap<String, (String, PathBuf, String)>,
}

/// Arguments passed between various download and adaptation methods.
#[derive(Clone)]
pub struct DownloadArgs {
    client: Arc<Client>,
    url_str: String,
    file_path: PathBuf,
    asset_dir: PathBuf,
    max_depth: usize,
    mp: Option<Arc<MultiProgress>>,
    channel: Option<Sender<DownloadInfo>>,
}
impl DownloadArgs {
    pub fn with_url_str(self, url_str: &str) -> Self {
        Self {
            url_str: url_str.to_string(),
            ..self
        }
    }
    pub fn with_file_path(self, file_path: impl Into<PathBuf>) -> Self {
        Self {
            file_path: file_path.into(),
            ..self
        }
    }
    pub fn with_asset_dir(self, asset_dir: impl Into<PathBuf>) -> Self {
        Self {
            asset_dir: asset_dir.into(),
            ..self
        }
    }
    pub fn with_max_depth(self, max_depth: usize) -> Self {
        Self { max_depth, ..self }
    }
    pub fn with_mp(self, mp: Option<Arc<MultiProgress>>) -> Self {
        Self { mp, ..self }
    }
    pub fn with_channel(self, channel: Option<Sender<DownloadInfo>>) -> Self {
        Self { channel, ..self }
    }
}
