# forum-saver

forum-saver 是一个用于将论坛帖子完整保存至本地的工具，包含网页和其中的图片、字体、CSS、JS 等资源文件，可以实现断网下的本地浏览。

## 特点

- **多论坛支持**：支持 Discuz、NGA、V2EX 等多种论坛系统，且易于扩展
- **递归遍历资源文件**：自动下载并本地化帖子中的图片、CSS、JavaScript 等资源文件
- **异步下载**：使用 Tokio 实现异步并发下载，提高效率
- **进度条与日志**：实时显示下载进度和详细日志信息
- **多种方式用户登录**：支持游客模式、从浏览器获取 Cookie、直接提供 Cookie 字符串、用户名密码登录等多种认证方式
- **国际化支持**：内置中英文语言支持，可自动检测系统语言
- **本地阅读优化**：支持删除广告、删除用户信息，并添加键盘左右箭头翻页支持

## 支持的论坛

- **Discuz**：当前支持 Stage1st、Chiphell、TGFC、77bike、lgqmonline，其余基于 Discuz 的论坛可以自行配置 config.toml 实现支持，配置方法参考 config.toml 中的注释。
- **NGA**：支持 NGA 论坛的帖子下载
- **V2EX**：支持 V2EX 论坛的帖子下载

后续计划扩展其他论坛系统，如 Discourse 等。

## 使用方式

### 安装

1. 源代码安装

需要本地安装 Rust 开发环境，推荐使用 [rustup](https://rustup.rs/) 安装。

```bash
# 克隆仓库
git clone https://github.com/shuangluoxss/forum-saver.git

# 进入目录
cd forum-saver

# 构建项目
cargo build --release

# 运行
./target/release/forum-saver
```

2. 从 github releases 下载预编译二进制文件

   1. 访问 [forum-saver releases](https://github.com/shuangluoxss/forum-saver/releases) 页面
   2. 下载适合您操作系统的预编译二进制文件（如 forum-saver-x86_64-unknown-linux-gnu）
   3. 解压文件（如 forum-saver-x86_64-unknown-linux-gnu.tar.gz）
   4. 运行解压后的二进制文件（如 ./forum-saver）

### 配置

1. **生成配置文件**

```bash
# 生成默认配置文件到 ~/forum-saver.toml
forum-saver gen-config

# 或指定输出路径
forum-saver gen-config config.toml
```

2. **编辑配置文件**

配置文件采用 TOML 格式，主要包含以下部分：

- 基本设置：存储目录、并发下载数、语言偏好等
- 论坛配置：每个论坛的名称、基础 URL、认证方式等

详细配置说明请参考生成的配置文件中的注释。

### 下载帖子

1. **下载单个帖子**

```bash
# 使用默认配置文件
forum-saver "https://example.com/thread/123"

# 使用指定配置文件
forum-saver -c config.toml "https://example.com/thread/123"
```

2. **批量下载多个帖子**

创建一个文本文件（如 urls.txt），每行一个帖子 URL：

```
# urls.txt
https://forum1.com/thread/123
https://www.forum2.com/thread/456
https://www.forum3.com/read.php?tid=789
```

然后执行：

```bash
# 使用默认配置文件
forum-saver -i urls.txt

# 使用指定配置文件
forum-saver -c config.toml -i urls.txt
```

### 认证方式

部分论坛可能需要登录才能访问完整内容，支持以下认证方式：

- **Guest**：游客模式，无需登录
- **CookieFromBrowser**：从浏览器获取 Cookie，支持 Chrome、Firefox、Edge、Opera、Safari
- **CookieString**：直接提供 Cookie 字符串
- **UsernamePassword**：用户名密码登录

### 存储结构

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

- **支持更多论坛**：计划扩展支持 Discourse 等其他论坛系统
- **支持仅下载部分页面**：允许用户指定只下载帖子的前几页或特定页面范围
- **失效楼层处理**：优化对已删除或被屏蔽楼层的处理
- **图形界面**：开发简单的图形界面，方便用户操作

## 注意事项

- 为避免触发论坛的访问限制，建议在配置文件中设置合理的页面下载间隔
- 部分论坛可能有防爬机制，可能需要使用登录方式获取完整内容
- 下载大量内容时请遵守相关论坛的使用规定，避免对服务器造成过大压力
