export interface DownloaderConfig {
  forums: ForumConfig[];
  downloadable_attrs: string[];
  downloadable_extensions: string[];
  max_path_length: number;
  path_hash_length: number;
  max_depth: number;
  store_dir: string;
  semaphore_count: number;
  language?: string;
  strategy?: DownloadStrategy;
}

export type DownloadStrategy = 'from-start' | 'resume-latest';

export type ForumConfig = 
  | { Discuz: DiscuzForumConfig }
  | { V2ex: V2exForumConfig }
  | { NGA: NGAForumConfig };

export interface DiscuzForumConfig {
  name: string;
  base_url: string;
  auth_method: AuthMethod;
  remove_ads?: boolean;
  remove_user_info?: boolean;
  remove_reply_box?: boolean;
  interval_ms?: number;
  thread_url_template?: string;
  selectors?: DiscuzSelectors;
}

export interface DiscuzSelectors {
  thread_title: string;
  pg_divs: string;
  pgbtn: string;
  pn_input: string;
  username: string;
  user_info: string;
  login_box: string;
  reply_box: string;
  ads: string;
  charset: string;
}

export interface NGAForumConfig {
  name: string;
  base_url: string;
  auth_method: AuthMethod;
  remove_ads?: boolean;
  remove_user_info?: boolean;
  remove_reply_box?: boolean;
  interval_ms?: number;
}

export interface V2exForumConfig {
  name: string;
  base_url: string;
  auth_method: AuthMethod;
  remove_ads?: boolean;
  remove_user_info?: boolean;
  remove_reply_box?: boolean;
  interval_ms?: number;
}

export type FormFieldType = 'input' | 'password' | 'textarea' | 'number' | 'switch' | 'select' | 'tags' | 'auth-method';

export interface FormField {
  key: string;
  label: string;
  type: FormFieldType;
  advanced?: boolean;
  span?: number;
  defaultValue?: any;
  placeholder?: string;
  options?: { label: string; value: any }[];
  min?: number;
  max?: number;
}

export interface FormGroupConfig {
  name?: string; // 可为空，为空时不显示标题
  fields: FormField[];
  collapsed?: boolean;
  collapsible?: boolean;
}

export interface FormSchema {
  groups: FormGroupConfig[];
  labelPlacement?: 'top' | 'left';
}

export type AuthMethod = 
  | { CookieString: string }
  | { CookieFromBrowser: SupportedBrowser }
  | { UsernamePassword: { username: string; password: string } }
  | "Guest";

export type SupportedBrowser = "Chrome" | "Firefox" | "Edge" | "Opera" | "Safari";
