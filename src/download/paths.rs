use std::path::{Path, PathBuf};

use super::models::ContentItem;

pub fn normalize_repo_path(path: &str) -> String {
    let normalized = path.trim();
    if matches!(normalized, "" | "." | "/") {
        String::new()
    } else {
        normalized.trim_matches('/').to_string()
    }
}

pub fn quote_repo_path(path: &str) -> String {
    normalize_repo_path(path)
        .split('/')
        .filter(|segment| !segment.is_empty())
        .map(urlencoding::encode)
        .map(|segment| segment.into_owned())
        .collect::<Vec<_>>()
        .join("/")
}

pub fn join_proxy_url(proxy_base: &str, target_url: &str) -> String {
    format!("{}/{}", proxy_base.trim_end_matches('/'), target_url)
}

pub fn redact_url_for_display(url: &str) -> String {
    let Some(scheme_separator) = url.find("://") else {
        return url.to_string();
    };

    let scheme_end = scheme_separator + 3;
    let authority_end = url[scheme_end..]
        .find(['/', '?', '#'])
        .map(|index| scheme_end + index)
        .unwrap_or(url.len());
    let authority = &url[scheme_end..authority_end];
    let Some(user_info_end) = authority.find('@') else {
        return url.to_string();
    };

    format!(
        "{}***@{}",
        &url[..scheme_end],
        &url[scheme_end + user_info_end + 1..]
    )
}

pub fn build_contents_api_url(
    api_base: &str,
    repo: &str,
    remote_path: &str,
    git_ref: Option<&str>,
) -> String {
    let quoted_path = quote_repo_path(remote_path);
    let mut url = format!("{}/repos/{}/contents", api_base.trim_end_matches('/'), repo);
    if !quoted_path.is_empty() {
        url.push('/');
        url.push_str(&quoted_path);
    }
    if let Some(git_ref) = git_ref.filter(|value| !value.trim().is_empty()) {
        let separator = if url.contains('?') { '&' } else { '?' };
        url.push(separator);
        url.push_str("ref=");
        url.push_str(&urlencoding::encode(git_ref));
    }
    url
}

pub fn format_remote_path(remote_path: &str) -> String {
    let normalized = normalize_repo_path(remote_path);
    if normalized.is_empty() {
        "/".to_string()
    } else {
        normalized
    }
}

pub fn relative_item_path(root_remote_path: &str, item_path: &str) -> String {
    let normalized_root = normalize_repo_path(root_remote_path);
    if normalized_root.is_empty() {
        return item_path.to_string();
    }

    let prefix = format!("{}/", normalized_root);
    item_path
        .strip_prefix(&prefix)
        .unwrap_or(item_path)
        .to_string()
}

pub(super) fn choose_file_target(
    local_target: &Path,
    remote_path: &str,
    item: &ContentItem,
) -> PathBuf {
    let filename = item
        .name
        .clone()
        .unwrap_or_else(|| file_name_from_remote_path(remote_path));
    if local_target.exists() && local_target.is_dir() {
        local_target.join(filename)
    } else {
        local_target.to_path_buf()
    }
}

pub fn choose_directory_target(local_target: &Path, remote_path: &str) -> PathBuf {
    let normalized_path = normalize_repo_path(remote_path);
    let directory_name = Path::new(&normalized_path)
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("");

    if directory_name.is_empty() {
        return local_target.to_path_buf();
    }

    if local_target
        .file_name()
        .and_then(|value| value.to_str())
        .is_some_and(|name| name == directory_name)
    {
        local_target.to_path_buf()
    } else {
        local_target.join(directory_name)
    }
}

pub(super) fn file_name_from_remote_path(remote_path: &str) -> String {
    Path::new(remote_path)
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("downloaded-file")
        .to_string()
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    #[test]
    fn directory_target_reuses_existing_suffix() {
        let local = PathBuf::from("/tmp/src");
        assert_eq!(choose_directory_target(&local, "src"), local);
    }

    #[test]
    fn directory_target_appends_directory_name() {
        let local = PathBuf::from("/tmp/downloads");
        assert_eq!(
            choose_directory_target(&local, "src"),
            PathBuf::from("/tmp/downloads/src")
        );
    }

    #[test]
    fn relative_item_path_strips_root_prefix() {
        assert_eq!(
            relative_item_path("src", "src/nested/lib.rs"),
            "nested/lib.rs".to_string()
        );
    }

    #[test]
    fn redacts_credentials_in_display_urls() {
        assert_eq!(
            redact_url_for_display("https://user:secret@example.com:8443/path"),
            "https://***@example.com:8443/path"
        );
        assert_eq!(
            redact_url_for_display("https://gh-proxy.com/https://api.github.com"),
            "https://gh-proxy.com/https://api.github.com"
        );
    }
}
