#[derive(Debug)]
pub struct ThreadInfo {
    pub title: String,
    pub thread_id: String,
    pub total_pages: usize,
    pub current_pn: usize,
    pub page_size: usize,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type")]
pub enum DownloadInfo {
    // 文本信息
    Text { message: String, level: String },
    // 进度信息
    Progress { current: u64, total: u64 },
}

impl DownloadInfo {
    pub fn new_text(message: String, level: String) -> Self {
        Self::Text { message, level }
    }
    pub fn new_info(message: String) -> Self {
        Self::Text {
            message,
            level: "info".into(),
        }
    }
    pub fn new_error(message: String) -> Self {
        Self::Text {
            message,
            level: "error".into(),
        }
    }
    pub fn new_progress(current: u64, total: u64) -> Self {
        Self::Progress { current, total }
    }
}
