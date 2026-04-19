use std::ffi::OsString;

use clap::ValueEnum;

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum Language {
    En,
    Zh,
}

impl Language {
    pub fn detect(
        explicit: Option<Language>,
        lc_all: Option<&str>,
        lc_messages: Option<&str>,
        lang: Option<&str>,
    ) -> Self {
        explicit.unwrap_or_else(|| {
            let locale = first_non_empty([lc_all, lc_messages, lang]);
            if locale.map(locale_indicates_chinese).unwrap_or(false) {
                Language::Zh
            } else {
                Language::En
            }
        })
    }

    pub fn is_chinese(self) -> bool {
        matches!(self, Language::Zh)
    }
}

pub fn detect_language_from_args_and_env(
    args: &[OsString],
    lc_all: Option<&str>,
    lc_messages: Option<&str>,
    lang: Option<&str>,
) -> Language {
    Language::detect(parse_language_override(args), lc_all, lc_messages, lang)
}

fn parse_language_override(args: &[OsString]) -> Option<Language> {
    let mut iter = args.iter().skip(1);
    while let Some(arg) = iter.next() {
        let value = arg.to_string_lossy();
        if let Some(raw) = value.strip_prefix("--lang=") {
            return parse_language_value(raw);
        }
        if value == "--lang" {
            if let Some(next) = iter.next() {
                return parse_language_value(&next.to_string_lossy());
            }
        }
    }
    None
}

fn parse_language_value(value: &str) -> Option<Language> {
    if value.eq_ignore_ascii_case("zh") {
        Some(Language::Zh)
    } else if value.eq_ignore_ascii_case("en") {
        Some(Language::En)
    } else {
        None
    }
}

fn first_non_empty<const N: usize>(values: [Option<&str>; N]) -> Option<&str> {
    values
        .into_iter()
        .flatten()
        .map(str::trim)
        .find(|value| !value.is_empty())
}

fn locale_indicates_chinese(locale: &str) -> bool {
    locale.to_ascii_lowercase().contains("zh")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn language_defaults_to_english_when_locale_is_not_chinese() {
        assert_eq!(
            Language::detect(None, None, None, Some("en_US.UTF-8")),
            Language::En
        );
    }

    #[test]
    fn language_switches_to_chinese_from_locale() {
        assert_eq!(
            Language::detect(None, None, None, Some("zh_CN.UTF-8")),
            Language::Zh
        );
    }

    #[test]
    fn explicit_language_overrides_locale() {
        assert_eq!(
            Language::detect(Some(Language::En), Some("zh_CN.UTF-8"), None, None),
            Language::En
        );
    }
}
