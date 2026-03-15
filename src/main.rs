use clap::Parser;
use forum_saver::cli::Cli;
use forum_saver::core::{Downloader, DownloaderConfig};
use forum_saver::i18n::I18n;
use log::{error, info};
use std::{fs, io::Write};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::builder()
        .filter_module("forum_saver", log::LevelFilter::Info)
        .format(|buf, record| {
            let local_time = chrono::Local::now();
            let level_style = buf.default_level_style(record.level());
            writeln!(
                buf,
                "[{} {}{:5}{}] {}",
                local_time.format("%Y-%m-%d %H:%M:%S%.3f"),
                level_style.render(),
                record.level(),
                level_style.render_reset(),
                record.args()
            )
        })
        .init();
    let i18n = I18n::new(None);
    let cli = Cli::parse();

    if cli.command.is_none() {
        let config_path = cli.config_path.unwrap_or_else(|| {
            let home_dir = dirs::home_dir().expect("Failed to get home directory");
            home_dir.join("forum-saver.toml")
        });

        // 检查配置文件是否存在
        if !config_path.exists() {
            return Err(anyhow::anyhow!(
                "{}",
                i18n.t(
                    "config-file-not-found",
                    Some(&[("path", &config_path.display().to_string())])
                )
            ));
        }

        info!(
            "{}",
            i18n.t(
                "loading-config",
                Some(&[("path", &config_path.display().to_string())])
            )
        );

        let downloader_config = DownloaderConfig::from_toml_file(&config_path)?;
        let downloader = Downloader::from_config(downloader_config)?;

        let forums: Vec<String> = downloader
            .supported_forums()
            .iter()
            .map(|f| f.to_string())
            .collect();
        info!(
            "{}",
            i18n.t("supported-forums", Some(&[("forums", &forums.join(", "))]))
        );

        if let Some(input_file) = cli.input_file {
            // 检查输入文件是否存在
            if !input_file.exists() {
                return Err(anyhow::anyhow!(
                    "{}",
                    i18n.t(
                        "config-file-not-found",
                        Some(&[("path", &input_file.display().to_string())])
                    )
                ));
            }

            let content = fs::read_to_string(&input_file)?;
            let urls: Vec<String> = content
                .lines()
                .map(|line| line.trim().to_string())
                .filter(|line| !line.is_empty() && !line.starts_with('#'))
                .collect();

            info!(
                "{}",
                i18n.t("total-urls", Some(&[("count", &urls.len().to_string())]))
            );

            for url in urls {
                if let Err(e) = downloader.download_thread(&url).await {
                    error!(
                        "{}",
                        i18n.t(
                            "failed-download-url",
                            Some(&[("url", &url), ("error", &format!("{:?}", e))])
                        )
                    );
                }
            }
        } else if let Some(thread_url) = cli.thread_url {
            if let Err(e) = downloader.download_thread(&thread_url).await {
                error!(
                    "{}",
                    i18n.t(
                        "failed-download-url",
                        Some(&[("url", &thread_url), ("error", &format!("{:?}", e))])
                    )
                );
            }
        };
    } else if let Some(command) = cli.command {
        match command {
            forum_saver::cli::Commands::GenConfig { output_path } => {
                // 获取输出文件路径
                let output_path = output_path.unwrap_or_else(|| {
                    // 默认路径：~/.forum-saver.toml
                    let home_dir = dirs::home_dir().expect("Failed to get home directory");
                    home_dir.join("forum-saver.toml")
                });

                // 读取 config_sample.toml 文件内容
                let sample_config_content = include_bytes!("../resources/config_sample.toml");
                if output_path.exists() {
                    error!(
                        "{}",
                        i18n.t(
                            "config-file-exists",
                            Some(&[("path", &output_path.display().to_string())])
                        )
                    );
                } else {
                    fs::write(&output_path, sample_config_content)?;
                    info!(
                        "{}",
                        i18n.t(
                            "generated-config",
                            Some(&[("path", &output_path.display().to_string())])
                        )
                    );
                }
            }
        }
    }

    Ok(())
}
