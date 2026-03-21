use crate::error::{Error, Result};
use reqwest::{Client, cookie::Jar};
use rookie;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use url::Url;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum SupportedBrowser {
    Chrome,
    Firefox,
    Edge,
    Opera,
    #[cfg(target_os = "macos")]
    Safari,
}

impl SupportedBrowser {
    pub fn extract_cookiejar(&self, domain: &str) -> Result<Jar> {
        let domains = vec![domain.to_string()];
        let domain_url: Url = format!("https://{domain}").parse()?;
        let cookie_jar = Jar::default();
        let cookies = match self {
            SupportedBrowser::Firefox => rookie::firefox(Some(domains)),
            SupportedBrowser::Chrome => rookie::chrome(Some(domains)),
            SupportedBrowser::Edge => rookie::edge(Some(domains)),
            SupportedBrowser::Opera => rookie::opera(Some(domains)),
            #[cfg(target_os = "macos")]
            SupportedBrowser::Safari => rookie::safari(Some(domains)),
        }
        .map_err(|e| {
            Error::BrowserCookie(format!("Browser: {self:?}, Domain: '{domain}', Error: {e}"))
        })?;
        for cookie in cookies {
            let cookie_str = format!(
                "{}={};Domain=.{domain}; Path=/",
                &cookie.name, &cookie.value
            );
            cookie_jar.add_cookie_str(&cookie_str, &domain_url);
        }
        Ok(cookie_jar)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum AuthMethod {
    CookieString(String),
    CookieFromBrowser(SupportedBrowser),
    UsernamePassword { username: String, password: String },
    Guest,
}
impl AuthMethod {
    pub fn from_cookie_str(cookie_str: impl ToString) -> Self {
        AuthMethod::CookieString(cookie_str.to_string())
    }
    pub fn from_browser(browser: SupportedBrowser) -> Self {
        AuthMethod::CookieFromBrowser(browser)
    }
    pub fn from_username_password(username: impl ToString, password: impl ToString) -> Self {
        AuthMethod::UsernamePassword {
            username: username.to_string(),
            password: password.to_string(),
        }
    }
}

impl AuthMethod {
    pub async fn generate_client(&self, domain: &str) -> Result<Client> {
        let user_agent = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36";
        match &self {
            AuthMethod::CookieString(cookie_str) => {
                let cookie_jar = Jar::default();
                let domain_url: Url = format!("https://{domain}").parse()?;
                for cookie in cookie_str.split(';') {
                    let cookie_with_domain = format!("{};Domain=.{domain}; Path=/", cookie.trim());
                    cookie_jar.add_cookie_str(&cookie_with_domain, &domain_url);
                }
                let client = Client::builder()
                    .cookie_provider(Arc::new(cookie_jar))
                    .user_agent(user_agent)
                    .build()?;
                Ok(client)
            }
            AuthMethod::CookieFromBrowser(browser) => {
                let cookie_jar = browser.extract_cookiejar(domain)?;
                let client = Client::builder()
                    .cookie_provider(Arc::new(cookie_jar))
                    .user_agent(user_agent)
                    .build()?;
                Ok(client)
            }
            // No cookie, return default client
            _ => {
                let client = Client::builder()
                    .cookie_store(true)
                    .user_agent(user_agent)
                    .build()?;
                Ok(client)
            }
        }
    }
}
impl Default for AuthMethod {
    fn default() -> Self {
        Self::Guest
    }
}
