use clap::{Parser, Subcommand};
use std::path::PathBuf;

const LONG_HELP: &'static str = r#"Examples:
  # Download a thread
  forum-saver "https://example.com/thread/123"
  forum-saver -c config.toml "https://example.com/thread/123"

  # Download multiple threads from file
  forum-saver -i urls.txt
  forum-saver -c config.toml -i urls.txt

  # Generate config file
  forum-saver gen-config
  forum-saver gen-config config.toml
  forum-saver g config.toml
"#;
#[derive(Parser, Debug)]
#[command(
    name = "forum-saver",
    about("Download thread from forum and make all resources localized"),
    author("shuangluoxss"),
    version("0.1.0"),
    styles(clap_cargo::style::CLAP_STYLING),
    arg_required_else_help(true),
    after_help(LONG_HELP)
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    // 当没有指定子命令时，直接作为下载命令的参数
    #[arg(
        short('c'),
        long("config"),
        help("Config file path. If not provided, will use ~/forum-saver.toml")
    )]
    pub config_path: Option<PathBuf>,
    #[arg(value_name("thread_url"), help("Thread URL to download"))]
    pub thread_url: Option<String>,
    #[arg(short('i'), long("input"), value_name("FILE"), help("File containing multiple thread URLs, one per line"))]
    pub input_file: Option<PathBuf>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Generate a sample configuration file
    #[command(alias("g"))]
    GenConfig {
        #[arg(
            value_name("output"),
            help("Output file path. If not provided, will use ~/forum-saver.toml")
        )]
        output_path: Option<PathBuf>,
    },
}
