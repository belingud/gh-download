use std::io;
use std::path::PathBuf;

use thiserror::Error;

use crate::i18n::Language;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserFacingError {
    pub title: String,
    pub reason: String,
    pub suggestions: Vec<String>,
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

    #[error("json parse failed: {0}")]
    Json(String),

    #[error("failed to write local path: {path}")]
    Io {
        path: PathBuf,
        #[source]
        source: io::Error,
    },

    #[error("invalid local path: {0}")]
    InvalidPath(String),
}

pub fn classify_error(
    error: &AppError,
    token_present: bool,
    language: Language,
) -> UserFacingError {
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
            title,
            reason: "连接 GitHub 或代理失败".to_string(),
            suggestions: vec![
                "检查当前网络是否可访问 GitHub".to_string(),
                "如果使用了 --proxy-base，请确认代理地址可访问".to_string(),
                "稍后重试，或提供 --token 降低匿名请求失败概率".to_string(),
            ],
        },
        (Language::En, AppError::Request { .. }) => UserFacingError {
            title,
            reason: "Failed to connect to GitHub or the configured proxy".to_string(),
            suggestions: vec![
                "Check whether the current network can reach GitHub".to_string(),
                "If you are using --proxy-base, verify that the proxy URL is reachable".to_string(),
                "Try again later, or provide --token to reduce anonymous request failures".to_string(),
            ],
        },
        (Language::Zh, AppError::Io { path, .. }) => UserFacingError {
            title,
            reason: format!("无法写入本地路径 {}", path.display()),
            suggestions: vec![
                "检查目标目录是否有写权限".to_string(),
                "确认磁盘空间充足，且目标文件未被其他程序占用".to_string(),
            ],
        },
        (Language::En, AppError::Io { path, .. }) => UserFacingError {
            title,
            reason: format!("Failed to write local path {}", path.display()),
            suggestions: vec![
                "Check whether the target directory is writable".to_string(),
                "Confirm that disk space is available and the file is not locked by another program".to_string(),
            ],
        },
        (Language::Zh, AppError::UnexpectedApiResponse) => UserFacingError {
            title,
            reason: "GitHub 返回了无法识别的响应格式".to_string(),
            suggestions: vec![
                "稍后重试，或检查仓库路径是否正确".to_string(),
                "如果问题持续出现，请附上命令和仓库信息进行排查".to_string(),
            ],
        },
        (Language::En, AppError::UnexpectedApiResponse) => UserFacingError {
            title,
            reason: "GitHub returned a response format that the CLI could not understand".to_string(),
            suggestions: vec![
                "Try again later, or verify that the repository path is correct".to_string(),
                "If the issue persists, capture the command and repository details for debugging".to_string(),
            ],
        },
        (Language::Zh, AppError::EmptyRepository) => UserFacingError {
            title,
            reason: "仓库参数不能为空".to_string(),
            suggestions: vec!["请按 OWNER/REPO 格式提供仓库参数".to_string()],
        },
        (Language::En, AppError::EmptyRepository) => UserFacingError {
            title,
            reason: "The repository argument cannot be empty".to_string(),
            suggestions: vec!["Provide the repository in OWNER/REPO format".to_string()],
        },
        (Language::Zh, AppError::EmptyRemotePath) => UserFacingError {
            title,
            reason: "远端路径参数不能为空".to_string(),
            suggestions: vec!["请提供仓库内文件或目录路径".to_string()],
        },
        (Language::En, AppError::EmptyRemotePath) => UserFacingError {
            title,
            reason: "The remote path argument cannot be empty".to_string(),
            suggestions: vec!["Provide a file or directory path inside the repository".to_string()],
        },
        (Language::Zh, AppError::MissingRepositoryPath) => UserFacingError {
            title,
            reason: "GitHub 返回的文件条目缺少仓库路径".to_string(),
            suggestions: vec!["稍后重试，或检查目标仓库路径是否正常".to_string()],
        },
        (Language::En, AppError::MissingRepositoryPath) => UserFacingError {
            title,
            reason: "GitHub returned a file entry without its repository path".to_string(),
            suggestions: vec!["Try again later, or verify that the target repository path is valid".to_string()],
        },
        (Language::Zh, AppError::Json(message)) => UserFacingError {
            title,
            reason: format!("解析 GitHub 响应失败：{}", message),
            suggestions: vec!["稍后重试，或检查代理返回内容是否被修改".to_string()],
        },
        (Language::En, AppError::Json(message)) => UserFacingError {
            title,
            reason: format!("Failed to parse the GitHub response: {}", message),
            suggestions: vec!["Try again later, or verify that the proxy response was not altered".to_string()],
        },
        (Language::Zh, AppError::InvalidPath(message)) => UserFacingError {
            title,
            reason: format!("本地路径无效：{}", message),
            suggestions: vec!["检查本地路径是否存在非法字符，或家目录是否可解析".to_string()],
        },
        (Language::En, AppError::InvalidPath(message)) => UserFacingError {
            title,
            reason: format!("Invalid local path: {}", message),
            suggestions: vec![
                "Check whether the local path contains invalid characters or whether the home directory can be resolved".to_string(),
            ],
        },
        (Language::Zh, AppError::HttpStatus { status, detail, .. }) => UserFacingError {
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
            title,
            reason: detail
                .clone()
                .unwrap_or_else(|| format!("GitHub request failed (HTTP {})", status)),
            suggestions: vec![
                "Try again later, or verify that the repository and path are correct".to_string(),
                "If the issue persists, check the proxy and authentication settings".to_string(),
            ],
        },
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
    }
}
