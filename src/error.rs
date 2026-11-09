use std::io;
use std::path::PathBuf;

use thiserror::Error;

use crate::i18n::Language;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserFacingError {
    pub code: &'static str,
    pub title: String,
    pub reason: String,
    pub suggestions: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FailedFile {
    pub path: String,
    pub local_path: String,
    pub reason: String,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct PartialStats {
    pub downloaded: usize,
    pub skipped_existing: usize,
    pub skipped_entries: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FailureDetails {
    pub failed: Vec<FailedFile>,
    pub stats: Option<PartialStats>,
}

pub fn failure_details(error: &AppError) -> FailureDetails {
    match error {
        AppError::DirectoryTransfer {
            failures, stats, ..
        } => FailureDetails {
            failed: failures.clone(),
            stats: Some(*stats),
        },
        _ => FailureDetails {
            failed: Vec::new(),
            stats: None,
        },
    }
}

#[derive(Debug, Error)]
pub enum AppError {
    #[error("repository cannot be empty")]
    EmptyRepository,

    #[error("remote path cannot be empty")]
    EmptyRemotePath,

    #[error("GitHub returned an unexpected response")]
    UnexpectedApiResponse,

    #[error("file entry is missing repository path")]
    MissingRepositoryPath,

    #[error("unsupported download URL: {0}")]
    InvalidUrl(String),

    #[error("HTTP {status} for {url}")]
    HttpStatus {
        status: u16,
        url: String,
        detail: Option<String>,
    },

    #[error("request failed: {message}")]
    Request {
        url: Option<String>,
        message: String,
    },

    #[error("transfer interrupted while reading response: {message}")]
    Transfer { path: PathBuf, message: String },

    #[error("directory transfer failed with {failed_count} failed file(s)")]
    DirectoryTransfer {
        failures: Vec<FailedFile>,
        stats: PartialStats,
        aborted_early: bool,
        failed_count: usize,
    },

    #[error("failed to download {remote_path} to {local_path}: {cause}")]
    Download {
        remote_path: String,
        local_path: PathBuf,
        attempts: Vec<String>,
        #[source]
        cause: Box<AppError>,
    },

    #[error("json parse failed: {0}")]
    Json(String),

    #[error("invalid configuration: {0}")]
    Config(String),

    #[error("failed to write local path: {path}")]
    Io {
        path: PathBuf,
        #[source]
        source: io::Error,
    },

    #[error("resolved local target is a directory: {path}")]
    TargetIsDirectory { path: PathBuf },

    #[error("invalid local path: {0}")]
    InvalidPath(String),
}

pub fn classify_error(
    error: &AppError,
    token_present: bool,
    language: Language,
) -> UserFacingError {
    if let AppError::Download {
        remote_path,
        attempts,
        cause,
        ..
    } = error
    {
        let mut classified = classify_error(cause, token_present, language);
        match language {
            Language::Zh => {
                classified.reason = format!("下载 {} 失败：{}", remote_path, classified.reason);
                if !attempts.is_empty() {
                    classified
                        .suggestions
                        .push(format!("已尝试的下载方式：{}", attempts.join("；")));
                }
            }
            Language::En => {
                classified.reason =
                    format!("Failed to download {}: {}", remote_path, classified.reason);
                if !attempts.is_empty() {
                    classified.suggestions.push(format!(
                        "Attempted download strategies: {}",
                        attempts.join("; ")
                    ));
                }
            }
        }
        return classified;
    }

    if let AppError::DirectoryTransfer {
        failures,
        aborted_early,
        ..
    } = error
    {
        let title = match language {
            Language::En => "✖ Directory download failed".to_string(),
            Language::Zh => "✖ 目录下载失败".to_string(),
        };
        let mut reason = match language {
            Language::En => format!("{} file(s) failed to download", failures.len()),
            Language::Zh => format!("{} 个文件下载失败", failures.len()),
        };
        for failure in failures.iter().take(5) {
            reason.push_str(&format!("\n- {}: {}", failure.path, failure.reason));
        }
        if failures.len() > 5 {
            reason.push_str(&match language {
                Language::En => "\n- ...",
                Language::Zh => "\n- ……",
            });
        }
        let mut suggestions = match language {
            Language::En => vec![
                "Fix the failing paths and rerun the download".to_string(),
                "Use --keep-going to attempt every file and collect all failures".to_string(),
                "Use --debug to inspect the attempted download strategies".to_string(),
            ],
            Language::Zh => vec![
                "修复失败的路径后重新执行下载".to_string(),
                "使用 --keep-going 尝试所有文件并收集全部失败".to_string(),
                "使用 --debug 查看已尝试的下载方式".to_string(),
            ],
        };
        if *aborted_early {
            suggestions.push(match language {
                Language::En => "Rerun with --keep-going to continue remaining files".to_string(),
                Language::Zh => "使用 --keep-going 重新执行以继续下载剩余文件".to_string(),
            });
        }
        return UserFacingError {
            code: "partial_transfer",
            title,
            reason,
            suggestions,
        };
    }

    let title = match language {
        Language::En => "✖ Download failed".to_string(),
        Language::Zh => "✖ 下载失败".to_string(),
    };

    match (language, error) {
        (Language::Zh, AppError::HttpStatus { status, detail, .. })
            if *status == 401 || *status == 403 || *status == 429 =>
        {
            let mut suggestions = vec![
                "设置环境变量 GITHUB_TOKEN 或 GH_TOKEN".to_string(),
                "或使用 --token <token> 重新执行".to_string(),
            ];
            if !token_present {
                suggestions.push("如果直连 GitHub 不稳定，可检查 --proxy-base 是否可访问".to_string());
            }
            UserFacingError {
                code: "authentication",
                title,
                reason: detail
                    .clone()
                    .unwrap_or_else(|| format!("GitHub 认证失败或触发限流（HTTP {}）", status)),
                suggestions,
            }
        }
        (Language::En, AppError::HttpStatus { status, detail, .. })
            if *status == 401 || *status == 403 || *status == 429 =>
        {
            let mut suggestions = vec![
                "Set GITHUB_TOKEN or GH_TOKEN in the environment".to_string(),
                "Or rerun with --token <token>".to_string(),
            ];
            if !token_present {
                suggestions.push(
                    "If direct GitHub access is unstable, verify that --proxy-base is reachable"
                        .to_string(),
                );
            }
            UserFacingError {
                code: "authentication",
                title,
                reason: detail.clone().unwrap_or_else(|| {
                    format!(
                        "GitHub authentication failed or the rate limit was hit (HTTP {})",
                        status
                    )
                }),
                suggestions,
            }
        }
        (Language::Zh, AppError::HttpStatus { status, detail, .. }) if *status == 404 => {
            UserFacingError {
                code: "not_found",
                title,
                reason: detail.clone().unwrap_or_else(|| {
                    "未找到指定的仓库、分支或远端路径，或者当前凭证无法访问该私有仓库".to_string()
                }),
                suggestions: vec![
                    "检查 owner/repo 是否正确".to_string(),
                    "检查 --ref 指向的分支、tag 或 commit 是否存在".to_string(),
                    "检查远端路径大小写是否正确".to_string(),
                    "如果是私有仓库，请提供 --token 或设置 GITHUB_TOKEN / GH_TOKEN".to_string(),
                ],
            }
        }
        (Language::En, AppError::HttpStatus { status, detail, .. }) if *status == 404 => {
            UserFacingError {
                code: "not_found",
                title,
                reason: detail.clone().unwrap_or_else(|| {
                    "The repository, ref, or remote path was not found, or the current credentials cannot access the private repository".to_string()
                }),
                suggestions: vec![
                    "Check whether owner/repo is correct".to_string(),
                    "Check whether --ref points to an existing branch, tag, or commit".to_string(),
                    "Check the remote path and its letter casing".to_string(),
                    "If this is a private repository, provide --token or set GITHUB_TOKEN / GH_TOKEN".to_string(),
                ],
            }
        }
        (Language::Zh, AppError::Request { .. }) => UserFacingError {
            code: "network",
            title,
            reason: "连接 GitHub 或代理失败".to_string(),
            suggestions: vec![
                "检查当前网络是否可访问 GitHub".to_string(),
                "如果使用了 --proxy-base，请确认代理地址可访问".to_string(),
                "稍后重试，或提供 --token 降低匿名请求失败概率".to_string(),
            ],
        },
        (Language::En, AppError::Request { .. }) => UserFacingError {
            code: "network",
            title,
            reason: "Failed to connect to GitHub or the configured proxy".to_string(),
            suggestions: vec![
                "Check whether the current network can reach GitHub".to_string(),
                "If you are using --proxy-base, verify that the proxy URL is reachable".to_string(),
                "Try again later, or provide --token to reduce anonymous request failures".to_string(),
            ],
        },
        (Language::Zh, AppError::Transfer { path, .. }) => UserFacingError {
            code: "network",
            title,
            reason: format!("下载写入前网络连接中断：{}", path.display()),
            suggestions: vec![
                "检查当前网络是否稳定，稍后重试".to_string(),
                "如果使用了 --proxy-base，请确认代理地址稳定".to_string(),
                "大文件下载失败时不会留下不完整的目标文件，可直接重试".to_string(),
            ],
        },
        (Language::En, AppError::Transfer { path, .. }) => UserFacingError {
            code: "network",
            title,
            reason: format!(
                "Network connection was interrupted before the download completed: {}",
                path.display()
            ),
            suggestions: vec![
                "Check whether the current network is stable and retry later".to_string(),
                "If you are using --proxy-base, verify that the proxy is stable".to_string(),
                "Failed downloads do not leave incomplete target files, so a retry is safe".to_string(),
            ],
        },
        (Language::Zh, AppError::Io { path, .. }) => UserFacingError {
            code: "io",
            title,
            reason: format!("无法写入本地路径 {}", path.display()),
            suggestions: vec![
                "检查目标目录是否有写权限".to_string(),
                "确认磁盘空间充足，且目标文件未被其他程序占用".to_string(),
            ],
        },
        (Language::En, AppError::Io { path, .. }) => UserFacingError {
            code: "io",
            title,
            reason: format!("Failed to write local path {}", path.display()),
            suggestions: vec![
                "Check whether the target directory is writable".to_string(),
                "Confirm that disk space is available and the file is not locked by another program".to_string(),
            ],
        },
        (Language::Zh, AppError::TargetIsDirectory { path }) => UserFacingError {
            code: "io",
            title,
            reason: format!("解析出的本地文件目标是已存在的目录：{}", path.display()),
            suggestions: vec![
                "为文件下载指定文件路径或不同的目录".to_string(),
                "如果要写入该目录，请使用目录下载或删除同名目录后重试".to_string(),
            ],
        },
        (Language::En, AppError::TargetIsDirectory { path }) => UserFacingError {
            code: "io",
            title,
            reason: format!(
                "The resolved local file target is an existing directory: {}",
                path.display()
            ),
            suggestions: vec![
                "Pass a file path or a different directory for this file download".to_string(),
                "If you intended a directory download, use a directory remote path or remove the conflicting directory first".to_string(),
            ],
        },
        (Language::Zh, AppError::UnexpectedApiResponse) => UserFacingError {
            code: "unexpected_response",
            title,
            reason: "GitHub 返回了无法识别的响应格式".to_string(),
            suggestions: vec![
                "稍后重试，或检查仓库路径是否正确".to_string(),
                "如果问题持续出现，请附上命令和仓库信息进行排查".to_string(),
            ],
        },
        (Language::En, AppError::UnexpectedApiResponse) => UserFacingError {
            code: "unexpected_response",
            title,
            reason: "GitHub returned a response format that the CLI could not understand".to_string(),
            suggestions: vec![
                "Try again later, or verify that the repository path is correct".to_string(),
                "If the issue persists, capture the command and repository details for debugging".to_string(),
            ],
        },
        (Language::Zh, AppError::InvalidUrl(url)) => UserFacingError {
            code: "unexpected_response",
            title,
            reason: format!("GitHub 返回了不支持的下载地址：{}", url),
            suggestions: vec![
                "检查 --api-base 或代理配置是否返回了异常地址".to_string(),
                "重试，或联系代理服务提供方".to_string(),
            ],
        },
        (Language::En, AppError::InvalidUrl(url)) => UserFacingError {
            code: "unexpected_response",
            title,
            reason: format!("GitHub returned an unsupported download URL: {}", url),
            suggestions: vec![
                "Check whether --api-base or the configured proxy returned an unexpected URL".to_string(),
                "Retry, or contact the proxy provider".to_string(),
            ],
        },
        (Language::Zh, AppError::EmptyRepository) => UserFacingError {
            code: "validation",
            title,
            reason: "仓库参数不能为空".to_string(),
            suggestions: vec!["请按 OWNER/REPO 格式提供仓库参数".to_string()],
        },
        (Language::En, AppError::EmptyRepository) => UserFacingError {
            code: "validation",
            title,
            reason: "The repository argument cannot be empty".to_string(),
            suggestions: vec!["Provide the repository in OWNER/REPO format".to_string()],
        },
        (Language::Zh, AppError::EmptyRemotePath) => UserFacingError {
            code: "validation",
            title,
            reason: "远端路径参数不能为空".to_string(),
            suggestions: vec!["请提供仓库内文件或目录路径".to_string()],
        },
        (Language::En, AppError::EmptyRemotePath) => UserFacingError {
            code: "validation",
            title,
            reason: "The remote path argument cannot be empty".to_string(),
            suggestions: vec!["Provide a file or directory path inside the repository".to_string()],
        },
        (Language::Zh, AppError::MissingRepositoryPath) => UserFacingError {
            code: "unexpected_response",
            title,
            reason: "GitHub 返回的文件条目缺少仓库路径".to_string(),
            suggestions: vec!["稍后重试，或检查目标仓库路径是否正常".to_string()],
        },
        (Language::En, AppError::MissingRepositoryPath) => UserFacingError {
            code: "unexpected_response",
            title,
            reason: "GitHub returned a file entry without its repository path".to_string(),
            suggestions: vec!["Try again later, or verify that the target repository path is valid".to_string()],
        },
        (Language::Zh, AppError::Json(message)) => UserFacingError {
            code: "unexpected_response",
            title,
            reason: format!("解析 GitHub 响应失败：{}", message),
            suggestions: vec!["稍后重试，或检查代理返回内容是否被修改".to_string()],
        },
        (Language::En, AppError::Json(message)) => UserFacingError {
            code: "unexpected_response",
            title,
            reason: format!("Failed to parse the GitHub response: {}", message),
            suggestions: vec!["Try again later, or verify that the proxy response was not altered".to_string()],
        },
        (Language::Zh, AppError::Config(message)) => UserFacingError {
            code: "config",
            title,
            reason: format!("配置文件无效：{}", message),
            suggestions: vec![
                "检查 --config 指向的文件是否存在且可读".to_string(),
                "确认配置文件使用 TOML 格式，且只包含 token、api_base、proxy_base、prefix_mode、concurrency、lang".to_string(),
                "如果不想使用配置文件，请修正该文件或移除 --config".to_string(),
            ],
        },
        (Language::En, AppError::Config(message)) => UserFacingError {
            code: "config",
            title,
            reason: format!("Configuration file error: {}", message),
            suggestions: vec![
                "Check whether the file passed to --config exists and is readable".to_string(),
                "Confirm that the file uses TOML and only contains token, api_base, proxy_base, prefix_mode, concurrency, and lang".to_string(),
                "If you do not want to use a config file, fix it or remove --config".to_string(),
            ],
        },
        (Language::Zh, AppError::InvalidPath(message)) => UserFacingError {
            code: "validation",
            title,
            reason: format!("本地路径无效：{}", message),
            suggestions: vec!["检查本地路径是否存在非法字符，或家目录是否可解析".to_string()],
        },
        (Language::En, AppError::InvalidPath(message)) => UserFacingError {
            code: "validation",
            title,
            reason: format!("Invalid local path: {}", message),
            suggestions: vec![
                "Check whether the local path contains invalid characters or whether the home directory can be resolved".to_string(),
            ],
        },
        (Language::Zh, AppError::HttpStatus { status, detail, .. }) => UserFacingError {
            code: "http_error",
            title,
            reason: detail
                .clone()
                .unwrap_or_else(|| format!("GitHub 请求失败（HTTP {}）", status)),
            suggestions: vec![
                "稍后重试，或检查仓库与路径是否正确".to_string(),
                "如果问题持续出现，请确认代理和认证配置".to_string(),
            ],
        },
        (Language::En, AppError::HttpStatus { status, detail, .. }) => UserFacingError {
            code: "http_error",
            title,
            reason: detail
                .clone()
                .unwrap_or_else(|| format!("GitHub request failed (HTTP {})", status)),
            suggestions: vec![
                "Try again later, or verify that the repository and path are correct".to_string(),
                "If the issue persists, check the proxy and authentication settings".to_string(),
            ],
        },
        (Language::Zh, AppError::Download { .. })
        | (Language::En, AppError::Download { .. })
        | (Language::Zh, AppError::DirectoryTransfer { .. })
        | (Language::En, AppError::DirectoryTransfer { .. }) => {
            unreachable!("aggregated transfer failures are classified before the match")
        }
    }
}

pub fn classify_parse_error(error: &clap::Error, language: Language) -> UserFacingError {
    let detail = error.render().to_string();
    let (title, reason, suggestions) = match language {
        Language::Zh => (
            "✖ 参数错误".to_string(),
            detail,
            vec![
                "检查命令语法和必填位置参数".to_string(),
                "运行 gh-download --help 查看完整用法".to_string(),
            ],
        ),
        Language::En => (
            "✖ Invalid arguments".to_string(),
            detail,
            vec![
                "Check the command syntax and required positional arguments".to_string(),
                "Run gh-download --help for full usage".to_string(),
            ],
        ),
    };

    UserFacingError {
        code: "usage",
        title,
        reason,
        suggestions,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classify_rate_limit_error_suggests_token() {
        let error = AppError::HttpStatus {
            status: 403,
            url: "https://api.github.com".to_string(),
            detail: Some("API rate limit exceeded".to_string()),
        };
        let user_error = classify_error(&error, false, Language::En);
        assert!(
            user_error
                .suggestions
                .iter()
                .any(|item| item.contains("GITHUB_TOKEN"))
        );
        assert_eq!(user_error.code, "authentication");
    }

    #[test]
    fn classified_errors_have_stable_codes() {
        let not_found = AppError::HttpStatus {
            status: 404,
            url: "https://api.github.com".to_string(),
            detail: None,
        };
        assert_eq!(
            classify_error(&not_found, false, Language::En).code,
            "not_found"
        );

        let invalid_url = AppError::InvalidUrl("file:///tmp/x".to_string());
        assert_eq!(
            classify_error(&invalid_url, false, Language::En).code,
            "unexpected_response"
        );
    }

    #[test]
    fn download_wrapper_mentions_remote_path_and_attempts() {
        let error = AppError::Download {
            remote_path: "src/lib.rs".to_string(),
            local_path: PathBuf::from("/tmp/out/src/lib.rs"),
            attempts: vec![
                "direct raw: HTTP 503".to_string(),
                "raw api: HTTP 404".to_string(),
            ],
            cause: Box::new(AppError::HttpStatus {
                status: 404,
                url: "https://api.github.com/repos/o/r/contents/src/lib.rs".to_string(),
                detail: Some("Not Found".to_string()),
            }),
        };

        let classified = classify_error(&error, false, Language::En);
        assert_eq!(classified.code, "not_found");
        assert!(classified.reason.contains("src/lib.rs"));
        assert!(
            classified
                .suggestions
                .iter()
                .any(|item| item.contains("direct raw: HTTP 503"))
        );
    }
}
