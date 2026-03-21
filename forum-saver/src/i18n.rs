use fluent::{FluentArgs, FluentResource};
use fluent_bundle::FluentBundle;
use unic_langid::langid;

pub struct I18n {
    bundle: FluentBundle<FluentResource>,
}
const EN: &str = include_str!("../resources/i18n/en-US.ftl");
const ZH: &str = include_str!("../resources/i18n/zh-CN.ftl");
impl I18n {
    pub fn new(language: Option<&str>) -> Self {
        let lang = if let Some(lang) = language {
            lang.to_string()
        } else {
            sys_locale::get_locale().unwrap_or("en".to_string())
        };
        let (lang_id, content) = if lang.to_lowercase().starts_with("zh") {
            (langid!("zh-CN"), ZH)
        } else {
            (langid!("en-US"), EN)
        };

        let mut bundle = FluentBundle::new(vec![lang_id]);
        let resource = FluentResource::try_new(content.to_string()).unwrap();
        bundle.add_resource(resource).unwrap();
        Self { bundle }
    }

    pub fn t(&self, key: &str, args: Option<&[(&str, &str)]>) -> String {
        match self.bundle.get_message(key) {
            Some(msg) => {
                let mut errors = vec![];
                let pattern = msg.value().unwrap_or_else(|| {
                    panic!("Message '{}' has no value", key);
                });

                let args = args.map(|args| {
                    let mut fluent_args = FluentArgs::new();
                    for (name, value) in args {
                        fluent_args.set(*name, *value);
                    }
                    fluent_args
                });

                self.bundle
                    .format_pattern(pattern, args.as_ref(), &mut errors)
                    .to_string()
            }
            None => key.to_string(),
        }
    }
}

// Convenience macro for logging
#[macro_export]
macro_rules! t {
    ($i18n:expr, $key:expr) => {
        $i18n.t($key, None)
    };
    ($i18n:expr, $key:expr, $($name:expr => $value:expr),*) => {
        $i18n.t($key, Some(&[$((stringify!($name), $value)),*]))
    };
}
