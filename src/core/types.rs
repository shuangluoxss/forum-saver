#[derive(Debug)]
pub struct ThreadInfo {
    pub title: String,
    pub thread_id: String,
    pub total_pages: usize,
    pub current_pn: usize,
    pub page_size: usize,
}
