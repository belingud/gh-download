use crate::cli::PrefixProxyMode;
use crate::error::AppError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum RawDownloadStrategy {
    DirectUrl,
    PrefixProxy,
    RawApi,
}

impl RawDownloadStrategy {
    pub(super) fn as_str(self) -> &'static str {
        match self {
            Self::DirectUrl => "direct-url",
            Self::PrefixProxy => "prefix-proxy",
            Self::RawApi => "raw-api",
        }
    }
}

pub(super) fn should_attempt_prefix_proxy(
    prefix_mode: PrefixProxyMode,
    proxy_base: &str,
    error: &AppError,
) -> bool {
    if prefix_mode != PrefixProxyMode::Fallback || proxy_base.trim().is_empty() {
        return false;
    }

    match error {
        AppError::HttpStatus { status, .. } => {
            matches!(*status, 403 | 429 | 500 | 502 | 503 | 504)
        }
        AppError::Request { .. } => true,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn http_error(status: u16) -> AppError {
        AppError::HttpStatus {
            status,
            url: "https://raw.githubusercontent.com".to_string(),
            detail: None,
        }
    }

    #[test]
    fn fallback_mode_retries_retryable_anonymous_raw_errors() {
        assert!(should_attempt_prefix_proxy(
            PrefixProxyMode::Fallback,
            "https://gh-proxy.com/",
            &http_error(503)
        ));
        assert!(should_attempt_prefix_proxy(
            PrefixProxyMode::Fallback,
            "https://gh-proxy.com/",
            &AppError::Request {
                url: None,
                message: "connection reset".to_string(),
            }
        ));
    }

    #[test]
    fn fallback_mode_skips_non_retryable_or_unconfigured_cases() {
        assert!(!should_attempt_prefix_proxy(
            PrefixProxyMode::Fallback,
            "https://gh-proxy.com/",
            &http_error(404)
        ));
        assert!(!should_attempt_prefix_proxy(
            PrefixProxyMode::Direct,
            "https://gh-proxy.com/",
            &http_error(503)
        ));
        assert!(!should_attempt_prefix_proxy(
            PrefixProxyMode::Fallback,
            "  ",
            &http_error(503)
        ));
    }
}
