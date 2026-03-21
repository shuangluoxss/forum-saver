import type { FormSchema, FormGroupConfig } from "./types";

const COMMON_FORUM_GROUPS: FormGroupConfig[] = [
  {
    name: "基础信息",
    collapsible: false,
    fields: [
      {
        key: "name",
        label: "名称",
        type: "input",
        placeholder: "例如: S1, NGA...",
        span: 8,
        defaultValue: ""
      },
      {
        key: "base_url",
        label: "基础 URL",
        type: "input",
        placeholder: "https://...",
        span: 16,
        defaultValue: "",
      },
    ],
  },
  {
    name: "登录认证",
    collapsible: true,
    collapsed: true,
    fields: [
      {
        key: "auth_method",
        label: "认证方式",
        type: "auth-method",
        span: 24,
        defaultValue: "Guest",
      },
    ],
  },
  {
    name: "内容清理",
    collapsible: true,
    collapsed: true,
    fields: [
      {
        key: "remove_ads",
        label: "移除广告",
        type: "switch",
        advanced: true,
        span: 6,
        defaultValue: true,
      },
      {
        key: "remove_user_info",
        label: "移除用户信息",
        type: "switch",
        advanced: true,
        span: 6,
        defaultValue: true,
      },
      {
        key: "remove_reply_box",
        label: "移除回复框",
        type: "switch",
        advanced: true,
        span: 6,
        defaultValue: true,
      },
    ],
  },
  {
    name: "抓取限制",
    collapsible: true,
    collapsed: true,
    fields: [
      {
        key: "interval_ms",
        label: "下载间隔 (ms)",
        type: "number",
        advanced: true,
        min: 0,
        span: 24,
        defaultValue: 1500,
      },
    ],
  },
];

export const DISCUZ_SCHEMA: FormSchema = {
  groups: [
    ...COMMON_FORUM_GROUPS,
    {
      name: "网址设置",
      collapsible: true,
      collapsed: true,
      fields: [
        {
          key: "thread_url_template",
          label: "URL 模板",
          type: "input",
          advanced: true,
          placeholder: "thread-{tid}-{pn}-1.html",
          span: 24,
          defaultValue: "thread-{tid}-{pn}-1.html",
        },
      ],
    },
    {
      name: "CSS 选择器",
      collapsible: true,
      collapsed: true,
      fields: [
        {
          key: "selectors.thread_title",
          label: "标题选择器",
          type: "input",
          advanced: true,
          span: 6,
          defaultValue: "h1",
        },
        {
          key: "selectors.pg_divs",
          label: "分页容器选择器",
          type: "input",
          advanced: true,
          span: 6,
          defaultValue: "div.pg",
        },
        {
          key: "selectors.pgbtn",
          label: "下一页按钮选择器",
          type: "input",
          advanced: true,
          span: 6,
          defaultValue: "div.pgbtn",
        },
        {
          key: "selectors.pn_input",
          label: "页码输入框选择器",
          type: "input",
          advanced: true,
          span: 6,
          defaultValue: "input.px[name='custompage']",
        },
        {
          key: "selectors.username",
          label: "用户名选择器",
          type: "input",
          advanced: true,
          span: 6,
          defaultValue: "strong.vwmy",
        },
        {
          key: "selectors.user_info",
          label: "用户信息选择器",
          type: "input",
          advanced: true,
          span: 6,
          defaultValue: "#um",
        },
        {
          key: "selectors.login_box",
          label: "登录框选择器",
          type: "input",
          advanced: true,
          span: 6,
          defaultValue: "div.y.pns",
        },
        {
          key: "selectors.reply_box",
          label: "回复框选择器",
          type: "input",
          advanced: true,
          span: 6,
          defaultValue: "#f_pst",
        },
        {
          key: "selectors.ads",
          label: "广告选择器",
          type: "input",
          advanced: true,
          span: 6,
          defaultValue: ".wp.a_h",
        },
        {
          key: "selectors.charset",
          label: "charset选择器",
          type: "input",
          advanced: true,
          span: 6,
          defaultValue: "meta[charset='gbk']",
        },
      ],
    },
  ],
};

export const NGA_SCHEMA: FormSchema = {
  groups: [...COMMON_FORUM_GROUPS],
};

export const V2EX_SCHEMA: FormSchema = {
  groups: [...COMMON_FORUM_GROUPS],
};

export const GLOBAL_SCHEMA: FormSchema = {
  groups: [
    {
      name: "存储与并发",
      collapsible: false,
      fields: [
        {
          key: "store_dir",
          label: "存储路径",
          type: "input",
          placeholder: "例如: ./data",
          span: 16,
          defaultValue: "./data",
        },
        {
          key: "semaphore_count",
          label: "并发线程数",
          type: "number",
          min: 1,
          max: 32,
          span: 8,
          defaultValue: 8,
        },
      ],
    },
    {
      name: "语言设置",
      collapsible: false,
      fields: [
        {
          key: "language",
          label: "语言 (Optional)",
          type: "select",
          span: 24,
          defaultValue: "zh",
          options: [
            { label: "中文 (zh)", value: "zh" },
            { label: "English (en)", value: "en" },
          ],
        },
      ],
    },
    {
      name: "抓取规则",
      collapsible: true,
      collapsed: false,
      fields: [
        {
          key: "downloadable_attrs",
          label: "可下载资源属性",
          type: "tags",
          advanced: true,
          span: 24,
          defaultValue: [
            "href",
            "src",
            "data-src",
            "file",
            "zoomfile",
            "poster",
            "style",
          ],
        },
        {
          key: "downloadable_extensions",
          label: "可下载扩展名",
          type: "tags",
          advanced: true,
          span: 24,
          defaultValue: [
            "png",
            "jpg",
            "jpeg",
            "gif",
            "svg",
            "webp",
            "tiff",
            "bmp",
            "js",
            "mjs",
            "css",
            "scss",
            "less",
            "woff",
            "woff2",
            "ttf",
            "eot",
            "otf",
            "mp4",
            "webm",
            "ogg",
            "mp3",
            "wav",
            "aac",
            "flac",
          ],
        },
      ],
    },
    {
      name: "高级路径",
      collapsible: true,
      collapsed: true,
      fields: [
        {
          key: "path_hash_length",
          label: "路径哈希长度",
          type: "number",
          advanced: true,
          min: 8,
          max: 32,
          span: 8,
          defaultValue: 16,
        },
        {
          key: "max_depth",
          label: "最大递归深度",
          type: "number",
          advanced: true,
          min: 1,
          max: 10,
          span: 8,
          defaultValue: 3,
        },
        {
          key: "max_path_length",
          label: "最大路径长度",
          type: "number",
          advanced: true,
          min: 100,
          max: 1024,
          span: 8,
          defaultValue: 240,
        },
      ],
    },
  ],
};

export const AUTH_METHOD_SCHEMAS: Record<string, FormGroupConfig[]> = {
  CookieString: [
    {
      fields: [
        {
          key: "CookieString",
          label: "Cookie 字符串",
          type: "textarea",
          span: 24,
          placeholder: "请输入完整 Cookie 字符串",
        },
      ],
    },
  ],
  CookieFromBrowser: [
    {
      fields: [
        {
          key: "CookieFromBrowser",
          label: "选择浏览器",
          type: "select",
          span: 24,
          options: [
            { label: "Chrome", value: "Chrome" },
            { label: "Firefox", value: "Firefox" },
            { label: "Edge", value: "Edge" },
            { label: "Opera", value: "Opera" },
            { label: "Safari", value: "Safari" },
          ],
        },
      ],
    },
  ],
  UsernamePassword: [
    {
      fields: [
        {
          key: "UsernamePassword.username",
          label: "用户名",
          type: "input",
          span: 12,
          placeholder: "请输入用户名",
        },
        {
          key: "UsernamePassword.password",
          label: "密码",
          type: "password",
          span: 12,
          placeholder: "请输入密码",
        },
      ],
    },
  ],
};
