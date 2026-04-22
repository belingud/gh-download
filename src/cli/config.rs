use std::env;
use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::error::AppError;
use crate::i18n::{Language, parse_language_override};

use super::types::PrefixProxyMode;

const DEFAULT_CONFIG_PATH: [&str; 3] = [".config", "gh-download", "config.toml"];

#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FileConfig {
    pub token: Option<String>,
    pub api_base: Option<String>,
    pub proxy_base: Option<String>,
    pub prefix_mode: Option<PrefixProxyMode>,
    pub concurrency: Option<usize>,
    pub lang: Option<Language>,
}

impl FileConfig {
    fn validate(self, path: &Path) -> Result<Self, AppError> {
        if matches!(self.concurrency, Some(0)) {
            return Err(config_error(
                path,
                "concurrency must be at least 1".to_string(),
            ));
        }
        Ok(self)
    }
}

pub(crate) fn detect_language_from_args_env_and_config(
    args: &[OsString],
    lc_all: Option<&str>,
    lc_messages: Option<&str>,
    lang: Option<&str>,
) -> Language {
    let config_path = parse_config_path_override(args);
    let config = load_active_config(config_path.as_deref()).ok().flatten();
    Language::detect(
        parse_language_override(args).or(config.as_ref().and_then(|value| value.lang)),
        lc_all,
        lc_messages,
        lang,
    )
}

pub(crate) fn load_active_config(
    explicit_path: Option<&Path>,
) -> Result<Option<FileConfig>, AppError> {
    let home = home_dir();
    load_active_config_with_home(explicit_path, home.as_deref())
}

pub(crate) fn load_active_config_with_home(
    explicit_path: Option<&Path>,
    home: Option<&Path>,
) -> Result<Option<FileConfig>, AppError> {
    let config_path = if let Some(path) = explicit_path {
        Some(expand_home(path)?)
    } else {
        default_config_path(home)
            .filter(|path| path.is_file())
            .map(Ok)
            .transpose()?
    };

    config_path
        .map(|path| read_config_file(&path).map(Some))
        .unwrap_or(Ok(None))
}

pub(crate) fn parse_config_path_override(args: &[OsString]) -> Option<PathBuf> {
    let mut iter = args.iter().skip(1);
    while let Some(arg) = iter.next() {
        let value = arg.to_string_lossy();
        if let Some(raw) = value.strip_prefix("--config=") {
            if !raw.is_empty() {
                return Some(PathBuf::from(raw));
            }
        }
        if value == "--config" {
            if let Some(next) = iter.next() {
                if next.to_string_lossy().starts_with('-') {
                    return None;
                }
                return Some(PathBuf::from(next));
            }
        }
    }
    None
}

pub(crate) fn expand_home(path: &Path) -> Result<PathBuf, AppError> {
    let raw = path.to_string_lossy();
    if raw == "~" {
        return home_dir()
            .ok_or_else(|| AppError::InvalidPath("failed to resolve home directory".to_string()));
    }
    if let Some(suffix) = raw.strip_prefix("~/").or_else(|| raw.strip_prefix("~\\")) {
        let home = home_dir()
            .ok_or_else(|| AppError::InvalidPath("failed to resolve home directory".to_string()))?;
        return Ok(home.join(suffix));
    }
    Ok(path.to_path_buf())
}

fn default_config_path(home: Option<&Path>) -> Option<PathBuf> {
    let home = home?;
    Some(
        DEFAULT_CONFIG_PATH
            .iter()
            .fold(home.to_path_buf(), |path, segment| path.join(segment)),
    )
}

fn read_config_file(path: &Path) -> Result<FileConfig, AppError> {
    let content = fs::read_to_string(path)
        .map_err(|error| config_error(path, format!("failed to read config file: {}", error)))?;
    toml::from_str::<FileConfig>(&content)
        .map_err(|error| config_error(path, format!("failed to parse config file: {}", error)))?
        .validate(path)
}

fn config_error(path: &Path, message: String) -> AppError {
    AppError::Config(format!("{} ({})", message, path.display()))
}

fn home_dir() -> Option<PathBuf> {
    env::var_os("HOME")
        .map(PathBuf::from)
        .or_else(|| env::var_os("USERPROFILE").map(PathBuf::from))
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;

    #[test]
    fn explicit_config_path_is_parsed_from_equals_form() {
        let args = vec![
            OsString::from("gh-download"),
            OsString::from("--config=/tmp/custom.toml"),
        ];

        assert_eq!(
            parse_config_path_override(&args),
            Some(PathBuf::from("/tmp/custom.toml"))
        );
    }

    #[test]
    fn default_config_file_is_loaded_from_home() {
        let temp = TempDir::new().expect("temp dir");
        let config_dir = temp.path().join(".config/gh-download");
        fs::create_dir_all(&config_dir).expect("create config dir");
        fs::write(
            config_dir.join("config.toml"),
            "lang = \"zh\"\nconcurrency = 6\n",
        )
        .expect("write config");

        let config = load_active_config_with_home(None, Some(temp.path()))
            .expect("config should load")
            .expect("config should exist");

        assert_eq!(config.lang, Some(Language::Zh));
        assert_eq!(config.concurrency, Some(6));
    }

    #[test]
    fn missing_default_config_file_is_ignored() {
        let temp = TempDir::new().expect("temp dir");

        let config =
            load_active_config_with_home(None, Some(temp.path())).expect("load should succeed");

        assert_eq!(config, None);
    }

    #[test]
    fn invalid_default_config_file_is_rejected() {
        let temp = TempDir::new().expect("temp dir");
        let config_dir = temp.path().join(".config/gh-download");
        fs::create_dir_all(&config_dir).expect("create config dir");
        fs::write(config_dir.join("config.toml"), "repo = \"owner/repo\"\n").expect("write config");

        let error = load_active_config_with_home(None, Some(temp.path()))
            .expect_err("invalid config should fail");

        assert!(matches!(error, AppError::Config(_)));
    }

    #[test]
    fn missing_explicit_config_file_is_rejected() {
        let temp = TempDir::new().expect("temp dir");
        let missing = temp.path().join("missing.toml");

        let error = load_active_config_with_home(Some(&missing), Some(temp.path()))
            .expect_err("missing config should fail");

        assert!(matches!(error, AppError::Config(_)));
    }

    #[test]
    fn config_language_is_used_before_locale() {
        let temp = TempDir::new().expect("temp dir");
        let config_path = temp.path().join("custom.toml");
        fs::write(&config_path, "lang = \"en\"\n").expect("write config");
        let args = vec![
            OsString::from("gh-download"),
            OsString::from("--config"),
            config_path.as_os_str().to_os_string(),
        ];

        let language =
            detect_language_from_args_env_and_config(&args, Some("zh_CN.UTF-8"), None, None);

        assert_eq!(language, Language::En);
    }
}
