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
    token_present: bool,
    proxy_base: &str,
    error: &AppError,
) -> bool {
    if prefix_mode != PrefixProxyMode::Fallback || token_present || proxy_base.trim().is_empty() {
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
