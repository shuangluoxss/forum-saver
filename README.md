# forum-saver

forum-saver 是一个用于将论坛帖子完整保存至本地的工具，包含网页和其中的图片、字体、CSS、JS 等资源文件，可实现断网下的本地浏览。

项目提供两种使用方式：

- **命令行工具**：适合熟悉命令行的用户，支持脚本自动化
- **图形界面应用**：基于 Tauri 开发的桌面应用，操作更直观

## 功能特点

- **多论坛支持**：支持 Discuz、NGA、V2EX 等多种论坛系统，且易于扩展
- **递归遍历资源文件**：自动下载并本地化帖子中的图片、CSS、JavaScript 等资源文件
- **异步并发下载**：使用 Tokio 实现异步并发下载，提高效率
- **进度条与日志**：实时显示下载进度和详细日志信息
- **多种认证方式**：支持游客模式、从浏览器获取 Cookie、直接提供 Cookie 字符串、用户名密码登录
- **国际化支持**：内置中英文语言支持，可自动检测系统语言
- **本地阅读优化**：支持删除广告、删除用户信息，并添加键盘左右箭头翻页支持

## 支持的论坛

- **Discuz**：当前支持 Stage1st、Chiphell、TGFC、77bike、lgqmonline，其余基于 Discuz 的论坛可以自行配置 `config.toml` 实现支持
- **NGA**：支持 NGA 论坛的帖子下载
- **V2EX**：支持 V2EX 论坛的帖子下载

后续计划扩展其他论坛系统，如 Discourse 等。

## 下载安装

### 从 Release 下载

访问 [Releases 页面](https://github.com/shuangluoxss/forum-saver/releases) 下载适合您系统的版本。由于 Release 中包含多个平台的文件，请参考下表选择正确的下载链接（将 `{version}` 替换为实际版本号，如 `0.2.0`）。

#### 命令行工具

| 平台    | 架构                  | 下载文件                                     |
| ------- | --------------------- | -------------------------------------------- |
| Windows | x86_64                | `forum-saver-{version}-windows-x86_64.zip`   |
| macOS   | Apple Silicon (M1-M5) | `forum-saver-{version}-macos-aarch64.tar.gz` |
| macOS   | Intel                 | `forum-saver-{version}-macos-x86_64.tar.gz`  |
| Linux   | x86_64                | `forum-saver-{version}-linux-x86_64.tar.gz`  |

#### 图形界面应用

| 平台    | 架构                  | 下载文件                                               |
| ------- | --------------------- | ------------------------------------------------------ |
| Windows | x86_64                | `forum-saver-tauri_{version}_x64-setup.exe` 或 `.msi`  |
| macOS   | Apple Silicon (M1-M5) | `forum-saver-tauri_{version}_aarch64.dmg`              |
| macOS   | Intel                 | `forum-saver-tauri_{version}_x64.dmg`                  |
| Linux   | x86_64                | `forum-saver-tauri_{version}_amd64.deb` 或 `.AppImage` |

### 从源码构建

#### 命令行工具

需要本地安装 Rust 开发环境，推荐使用 [rustup](https://rustup.rs/) 安装。

```bash
git clone https://github.com/shuangluoxss/forum-saver.git
cd forum-saver
cargo build --release
./target/release/forum-saver
```

#### 图形界面应用

需要安装 Node.js、pnpm 和 Rust 开发环境。

```bash
git clone https://github.com/shuangluoxss/forum-saver.git
cd forum-saver/tauri-app
pnpm install
pnpm tauri build
```

构建产物位于 `target/release/bundle/` 目录下，具体路径参见日志输出。

---

## 命令行工具使用说明

### 配置

1. **生成配置文件**

```bash
# 生成默认配置文件到 ~/.config/forum-saver/config.toml
forum-saver gen-config

# 或指定输出路径
forum-saver gen-config config.toml
```

2. **编辑配置文件**

配置文件采用 TOML 格式，主要包含以下部分：

- 基本设置：存储目录、并发下载数、语言偏好等
- 论坛配置：每个论坛的名称、基础 URL、认证方式等

详细配置说明请参考生成的配置文件中的注释, 或 [配置文件示例](./forum-saver/resources/config_sample.toml)。

### 下载帖子

1. **下载单个帖子**

```bash
# 使用默认配置文件
forum-saver "https://example.com/thread/123"

# 使用指定配置文件
forum-saver -c config.toml "https://example.com/thread/123"
```

2. **批量下载多个帖子**

创建一个文本文件（如 `urls.txt`），每行一个帖子 URL：

```
https://forum1.com/thread/123
https://www.forum2.com/thread/456
https://www.forum3.com/read.php?tid=789
```

然后执行：

```bash
forum-saver -i urls.txt
```

### 认证方式

部分论坛可能需要登录才能访问完整内容，支持以下认证方式：

- **Guest**：游客模式，无需登录
- **CookieFromBrowser**：从浏览器获取 Cookie，支持 Chrome、Firefox、Edge、Opera、Safari
- **CookieString**：直接提供 Cookie 字符串
- **UsernamePassword**：用户名密码登录

---

## 图形界面应用使用说明

图形界面应用基于 Tauri + Vue 3 开发，提供直观的操作界面。

### 功能介绍

应用包含两个主要页面：

1. **下载页面**
   - 输入帖子 URL 即可开始下载
   - 实时显示下载进度条
   - 显示详细的下载日志

2. **配置页面**
   - 全局配置：存储目录、并发数、语言等
   - 论坛配置：添加、编辑、删除论坛配置
   - 支持高级模式显示更多配置项
   - 点击右下角保存按钮，将配置自动保存至 `~/.config/forum-saver/config.toml`

### 自动更新

应用内置自动更新功能，启动时会自动检查新版本并提示更新。

---

## 存储结构

下载的内容会按照以下结构存储：

```
store_dir/
├── 论坛名称1/
│   ├── posts/         # 帖子 HTML 文件
│   └── assets/        # 资源文件（图片、CSS、JS 等）
├── 论坛名称2/
│   ├── posts/
│   └── assets/
└── ...
```

## 后续计划

- 支持更多论坛：计划扩展支持 Discourse 等其他论坛系统
- 支持仅下载部分页面：允许用户指定只下载帖子的前几页或特定页面范围
- 失效楼层处理：优化对已删除或被屏蔽楼层的处理

## 注意事项

- 为避免触发论坛的访问限制，建议在配置文件中设置合理的页面下载间隔
- 部分论坛可能有防爬机制，可能需要使用登录方式获取完整内容
- 下载大量内容时请遵守相关论坛的使用规定，避免对服务器造成过大压力

## 更新日志

### v0.2.0, 2026-03-21

- 新增基于 Tauri 的图形界面，支持可视化配置管理、实时下载进度显示、自动更新
- 修改配置文件默认路径为 `~/.config/forum-saver/config.toml`
- NGA帖子添加表情解析与下载

### v0.1.0, 2026-03-15

- 初版发布
