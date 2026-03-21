use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("HTTP status error: {0}")]
    HttpStatus(reqwest::StatusCode),
    #[error("Acquire error: {0}")]
    Acquire(#[from] tokio::sync::AcquireError),
    #[error("URL parse error: {0}")]
    ParseUrl(#[from] url::ParseError),
    #[error("Parse int error: {0}")]
    ParseInt(#[from] std::num::ParseIntError),
    #[error("InvalidForum {0}")]
    InvalidForum(String),
    #[error("Browser cookie error: {0}")]
    BrowserCookie(String),
    #[error("Login error: {0}")]
    Login(String),
    #[error("Node error: {0}")]
    ThreadInfo(String),
    #[error("HTML parse error: {0}")]
    HtmlParse(String),
    #[error("Path error: {0}")]
    Path(String),
    #[error("Browser error: {0}")]
    Browser(String),
    #[error("Anyhow error: {0}")]
    Anyhow(#[from] anyhow::Error),
    #[error("Toml deserialize error: {0}")]
    TomlDeserialize(#[from] toml::de::Error),
    #[error("Toml serialize error: {0}")]
    TomlSerialize(#[from] toml::ser::Error),
    #[error("Other error: {0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, Error>;
