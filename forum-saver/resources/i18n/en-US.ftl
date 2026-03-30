# Log messages
downloading-thread = Downloading thread: "{$url}"
thread-url = Thread URL: "{$url}"
username = Forum: {$forum}, Username: {$username}
not-login = Forum: {$forum}, Not login
started-downloading = Started downloading thread: "{$title}", {$pages} pages in total
resume-from-page = Found local pages, resuming from page {$page}...
error-fetching-page = Error fetching page {$page}: {$error}
thread-complete = Thread download complete. Saved to: "{$path}"
download-failed = Download thread failed: {$error}

# Progress bar messages
initializing = Initializing: Processing first page...
page-saved = Page {$current}/{$total} saved. Cooling down for {$ms}ms...
requesting-page = Requesting page {$current}/{$total}...
all-pages-fetched = All pages fetched. Merging content and generating index...
downloading-asset = Downloading asset: {$url}
interval-ms-zero = Interval set to 0, async fetching all pages (fast but risk to be banned)

# CLI messages
loading-config = Loading config file: {$path}
supported-forums = Supported forum: [{$forums}]
total-urls = Total {$count} URLs to download
failed-download-url = Failed to download thread: {$url} - {$error}
config-file-not-found = Config file not found: {$path}, you could run `forum-saver gen-config` to generate
generated-config = Generated sample config file: {$path}
config-file-exists = Config file exists, delete it or choose another path: {$path}