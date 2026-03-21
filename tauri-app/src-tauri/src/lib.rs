// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use forum_saver::core::{DownloadInfo, Downloader, DownloaderConfig};
use forum_saver::utils::sample_config_text;
use std::fs;
use std::path::{Path, PathBuf};
use tauri::Manager;
fn get_config_path(app: &tauri::AppHandle) -> PathBuf {
    let home = app
        .path()
        .home_dir()
        .expect("Could not find home directory");
    let config_dir = home.join(".config").join("forum-saver");
    fs::create_dir_all(&config_dir).expect("Could not create config directory");
    config_dir.join("config.toml")
}

#[tauri::command]
async fn download_thread(
    app: tauri::AppHandle,
    url: String,
    channel: tauri::ipc::Channel<DownloadInfo>,
) -> Result<(), String> {
    let config_path = get_config_path(&app);
    if !config_path.exists() {
        return Err("Config file not found. Please configure forums first.".to_string());
    }
    let config = load_config(app.clone()).await?;

    // 创建一个 oneshot 通道，用于将后台线程的最终结果传回给 Tauri
    let (res_tx, res_rx) = tokio::sync::oneshot::channel();

    // 开启一个独立的系统线程，专门处理 DOM 和并发下载
    std::thread::spawn(move || {
        // 构建一个“单线程”的 Tokio 运行时
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        // 使用 LocalSet 来运行无需 Send 的 future
        let local = tokio::task::LocalSet::new();

        let result = local.block_on(&rt, async move {
            let downloader = match Downloader::from_config(config) {
                Ok(d) => d,
                Err(e) => return Err(e.to_string()),
            };

            let (tx, mut rx) = tokio::sync::mpsc::channel(100);

            // 在单线程运行时中 spawn 一个本地任务转发消息
            tokio::task::spawn_local(async move {
                while let Some(info) = rx.recv().await {
                    let _ = channel.send(info);
                }
            });

            // 此时调用 download_thread，内部无论怎么跨越 await 传递 Rc 都不再会报错
            downloader
                .download_thread(&url, Some(tx))
                .await
                .map_err(|e| e.to_string())
        });

        let _ = res_tx.send(result);
    });

    // 挂起 Tauri command，等待后台线程执行完毕
    res_rx
        .await
        .map_err(|_| "Background thread panicked".to_string())?
}

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
async fn load_config(app: tauri::AppHandle) -> Result<DownloaderConfig, String> {
    let path = get_config_path(&app);
    if !path.exists() {
        // return Ok(DownloaderConfig::default());
        let sample_config_text = sample_config_text();
        fs::write(&path, sample_config_text).map_err(|e| e.to_string())?;
    }

    let content = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let config: DownloaderConfig = toml::from_str(&content).map_err(|e| e.to_string())?;
    Ok(config)
}

#[tauri::command]
async fn save_config(app: tauri::AppHandle, config: DownloaderConfig) -> Result<(), String> {
    let path = get_config_path(&app);
    let content = toml::to_string_pretty(&config).map_err(|e| e.to_string())?;
    fs::write(path, content).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
async fn export_config(path: &str, config: DownloaderConfig) -> Result<(), String> {
    let content = toml::to_string_pretty(&config).map_err(|e| e.to_string())?;
    fs::write(path, content).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
async fn import_config(path: &str) -> Result<DownloaderConfig, String> {
    let path = Path::new(path);
    if !path.exists() {
        return Err(format!("Config file not found: {}", path.display()));
    }

    let content = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let config: DownloaderConfig = toml::from_str(&content).map_err(|e| e.to_string())?;
    Ok(config)
}

#[tauri::command]
async fn generate_default_config() -> DownloaderConfig {
    DownloaderConfig::default()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            load_config,
            save_config,
            export_config,
            import_config,
            generate_default_config,
            download_thread
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
