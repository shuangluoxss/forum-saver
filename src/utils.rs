use crate::error::{Error, Result};
use kuchiki::{NodeRef, traits::TendrilSink};
use lazy_static::lazy_static;
use regex::Regex;

use sha2::{Digest, Sha256};
use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
};
use tokio::io::AsyncWriteExt;

lazy_static! {
    static ref INVALID_PATH_CHARACTERS: Regex = Regex::new(r#"[<>:"|?*]+"#).unwrap();
}
/// Clean path by replacing invalid characters
pub fn clean_path(path_str: &str, replace_char: &str) -> String {
    INVALID_PATH_CHARACTERS
        .replace_all(path_str, replace_char)
        .to_string()
}

use url::Url;
/// Convert URL to local file path
pub fn url_to_path(
    parsed_url: &Url,
    base_dir: &Path,
    default_filename: &str,
    max_path_length: usize,
    hash_length: usize,
) -> Result<PathBuf> {
    let netloc = parsed_url.host_str().unwrap_or("local");

    let path = percent_encoding::percent_decode_str(parsed_url.path())
        .decode_utf8()
        .map_err(|e| Error::Path(e.to_string()))?
        .replace('\\', "/")
        .trim_start_matches("/")
        .to_string();

    let (dirname, filename) = if let Some(last_slash) = path.rfind('/') {
        let (dir, file) = path.split_at(last_slash);
        (dir, &file[1..])
    } else {
        ("", path.as_str())
    };

    let filename = if filename.is_empty() {
        default_filename
    } else {
        filename
    };

    let safe_dirname = clean_path(dirname, "_");
    let safe_filename = clean_path(filename, "_");

    let mut full_path = PathBuf::new();
    full_path.push(base_dir);
    full_path.push(netloc);
    full_path.push(safe_dirname);
    full_path.push(safe_filename);

    // Handle path length limitation
    if full_path.to_string_lossy().len() < max_path_length {
        return Ok(full_path);
    }

    let suffix = full_path.extension().and_then(OsStr::to_str).unwrap_or("");

    if base_dir.to_string_lossy().len() + hash_length + suffix.len() > max_path_length - 1 {
        return Err(Error::Path("Path length exceeds limit".into()));
    }

    let path_hash = {
        let mut hasher = Sha256::new();
        hasher.update(full_path.to_string_lossy().as_bytes());
        let result = hasher.finalize();
        hex::encode(&result)[..hash_length].to_string()
    };

    let max_path_len = max_path_length - hash_length - suffix.len();
    let mut parent = full_path.parent().unwrap().to_path_buf();

    while parent.to_string_lossy().len() >= max_path_len {
        if let Some(p) = parent.parent() {
            parent = p.to_path_buf();
        } else {
            break;
        }
    }

    let mut new_filename = path_hash;
    if !suffix.is_empty() {
        new_filename.push('.');
        new_filename.push_str(suffix);
    }

    Ok(parent.join(new_filename))
}

pub async fn async_make_parent_dir(path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            tokio::fs::create_dir_all(parent).await?;
        }
    }
    Ok(())
}

pub fn create_node(html_text: &str, tag_name: &str) -> Result<NodeRef> {
    kuchiki::parse_html()
        .one(html_text)
        .select_first(tag_name)
        .map(|node| node.as_node().clone())
        .map_err(|_| Error::HtmlParse("无法解析为节点".into()))
}

pub fn generate_keyboard_nav_node(
    prev_url: Option<String>,
    next_url: Option<String>,
) -> Result<NodeRef> {
    let prev_url = prev_url.unwrap_or_default();
    let next_url = next_url.unwrap_or_default();

    create_node(
        &format!(
            r#"<script type="text/javascript">document.onkeyup = function (e) {{
            var keyCode = e.keyCode;
            if (keyCode === 37) {{ // LeftArrow
                if ("{prev_url}") {{window.location.href = "{prev_url}";}}
            }} else if (keyCode === 39) {{ // RightArrow
                if ("{next_url}") {{window.location.href = "{next_url}";}}
            }}
        }}</script>"#
        ),
        "script",
    )
}
pub async fn async_write_html(document: NodeRef, file_path: &Path) -> Result<()> {
    let mut writer = tokio::io::BufWriter::new(tokio::fs::File::create(file_path).await?);
    let mut buf = Vec::new();
    document.serialize(&mut buf)?;
    writer.write_all(&buf).await?;
    writer.flush().await?;
    Ok(())
}

pub async fn decode_response_text(response: reqwest::Response) -> Result<String> {
    let encoding = response
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .and_then(|content_type| {
            let parts: Vec<&str> = content_type.split(';').collect();
            parts.get(1).and_then(|param| {
                let param_parts: Vec<&str> = param.trim().split('=').collect();
                if param_parts.len() == 2 && param_parts[0] == "charset" {
                    Some(param_parts[1].trim().to_string())
                } else {
                    None
                }
            })
        });

    let bytes = response.bytes().await?;

    // Decode the content based on detected encoding
    let text = match encoding {
        Some(encoding_name) => {
            match encoding_rs::Encoding::for_label(encoding_name.as_bytes()) {
                Some(encoding) => {
                    let (decoded, _, _) = encoding.decode(&bytes);
                    decoded.to_string()
                }
                None => {
                    // Fallback to UTF-8 if encoding is not recognized
                    String::from_utf8_lossy(&bytes).to_string()
                }
            }
        }
        None => {
            // No encoding in headers, try to detect from content
            let (decoded, _, _) = encoding_rs::GBK.decode(&bytes);
            let gbk_result = decoded.to_string();

            // Check if GBK decoding makes sense (not too many replacement characters)
            let replacement_count = gbk_result.chars().filter(|&c| c == '�').count();
            let replacement_ratio = replacement_count as f64 / gbk_result.len() as f64;

            if replacement_ratio < 0.01 {
                // GBK decoding seems reasonable
                gbk_result
            } else {
                // Fallback to UTF-8
                String::from_utf8_lossy(&bytes).to_string()
            }
        }
    };
    Ok(text)
}
