# Log messages
downloading-thread = 正在下载帖子: "{$url}"
thread-url = 帖子链接: "{$url}"
username = 论坛: {$forum}，用户名: {$username}
not-login = 论坛: {$forum}，未登录
started-downloading = 开始下载帖子: "{$title}", 共 {$pages} 页
resume-from-page = 检测到本地已有页码，从第 {$page} 页开始下载...
error-fetching-page = 获取第 {$page} 页失败: {$error}
thread-complete = 帖子下载完成。保存到: "{$path}"
download-failed = 下载帖子失败: {$error}

# Progress bar messages
initializing = 初始化: 处理第一页...
page-saved = 第 {$current}/{$total} 页已保存。休眠 {$ms}ms...
requesting-page = 获取第 {$current}/{$total} 页...
all-pages-fetched = 所有页面已获取。合并内容并生成索引...
downloading-asset = 下载资源: {$url}
interval-ms-zero = 休眠间隔设为0，异步下载所有页面（速度快但易触发封禁）

# CLI messages
loading-config = 加载配置文件: {$path}
supported-forums = 支持的论坛: [{$forums}]
total-urls = 共 {$count} 个 URL 需要下载
failed-download-url = 下载帖子失败: {$url} - {$error}
config-file-not-found = 配置文件不存在: {$path}, 可以运行`forum-saver gen-config`命令生成
generated-config = 已生成示例配置文件: {$path}
config-file-exists = 配置文件已存在，请先删除或修改其他路径: {$path}
