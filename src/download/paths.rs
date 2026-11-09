use std::path::{Component, Path, PathBuf};

use crate::error::AppError;

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

pub const DEFAULT_RAW_BASE: &str = "https://raw.githubusercontent.com";

pub const DEFAULT_MEDIA_BASE: &str = "https://media.githubusercontent.com/media";

pub fn build_media_file_url(repo: &str, rev: &str, repo_path: &str) -> String {
    format!(
        "{}/{}/{}/{}",
        DEFAULT_MEDIA_BASE,
        repo.trim_matches('/'),
        rev.trim_matches('/'),
        quote_repo_path(repo_path)
    )
}

pub fn build_raw_file_url(repo: &str, rev: &str, repo_path: &str) -> String {
    format!(
        "{}/{}/{}/{}",
        DEFAULT_RAW_BASE,
        repo.trim_matches('/'),
        rev.trim_matches('/'),
        quote_repo_path(repo_path)
    )
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

pub(super) fn safe_directory_relative_item_path(
    root_remote_path: &str,
    item_path: &str,
) -> Result<String, AppError> {
    let relative_path = checked_relative_item_path(root_remote_path, item_path)?;
    validate_relative_output_path(&relative_path)?;
    Ok(relative_path)
}

fn checked_relative_item_path(root_remote_path: &str, item_path: &str) -> Result<String, AppError> {
    let normalized_root = normalize_repo_path(root_remote_path);
    if normalized_root.is_empty() {
        return Ok(item_path.to_string());
    }

    let prefix = format!("{}/", normalized_root);
    item_path
        .strip_prefix(&prefix)
        .map(ToOwned::to_owned)
        .ok_or_else(|| unsafe_directory_metadata_path(item_path))
}

fn validate_relative_output_path(relative_path: &str) -> Result<(), AppError> {
    if relative_path.is_empty() || relative_path.contains('\\') {
        return Err(unsafe_directory_metadata_path(relative_path));
    }

    let path = Path::new(relative_path);
    if path.is_absolute() {
        return Err(unsafe_directory_metadata_path(relative_path));
    }

    let mut has_component = false;
    for component in path.components() {
        match component {
            Component::Normal(_) => has_component = true,
            Component::CurDir
            | Component::ParentDir
            | Component::RootDir
            | Component::Prefix(_) => {
                return Err(unsafe_directory_metadata_path(relative_path));
            }
        }
    }

    if has_component {
        Ok(())
    } else {
        Err(unsafe_directory_metadata_path(relative_path))
    }
}

fn unsafe_directory_metadata_path(path: &str) -> AppError {
    AppError::InvalidPath(format!("unsafe directory metadata path: {}", path))
}

pub(super) fn choose_file_target(
    local_target: &Path,
    remote_path: &str,
    item: &ContentItem,
    dir_hint: bool,
) -> Result<PathBuf, AppError> {
    let filename = validated_metadata_file_name(item.name.as_deref(), remote_path)?;
    if local_target.exists() {
        if local_target.is_dir() {
            return Ok(local_target.join(filename));
        }
        if dir_hint {
            return Err(AppError::InvalidPath(format!(
                "local target has a trailing separator but exists as a file: {}",
                local_target.display()
            )));
        }
        return Ok(local_target.to_path_buf());
    }

    if dir_hint {
        Ok(local_target.join(filename))
    } else {
        Ok(local_target.to_path_buf())
    }
}

pub(super) fn validated_metadata_file_name(
    metadata_name: Option<&str>,
    remote_path: &str,
) -> Result<String, AppError> {
    if let Some(name) = metadata_name.filter(|name| is_safe_file_name(name)) {
        return Ok(name.to_string());
    }

    let fallback = file_name_from_remote_path(remote_path);
    if is_safe_file_name(&fallback) {
        Ok(fallback)
    } else {
        Err(AppError::InvalidPath(format!(
            "unsafe metadata file name {:?} for remote path {:?}",
            metadata_name.unwrap_or(""),
            remote_path
        )))
    }
}

fn is_safe_file_name(name: &str) -> bool {
    !name.is_empty()
        && name != "."
        && name != ".."
        && !name.contains('/')
        && !name.contains('\\')
        && !name.contains('\0')
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
    fn safe_directory_relative_item_path_accepts_nested_paths() {
        assert_eq!(
            safe_directory_relative_item_path("src", "src/nested/lib.rs")
                .expect("path should be valid"),
            "nested/lib.rs"
        );
    }

    #[test]
    fn safe_directory_relative_item_path_accepts_root_download_paths() {
        assert_eq!(
            safe_directory_relative_item_path("", "src/main.rs").expect("path should be valid"),
            "src/main.rs"
        );
    }

    #[test]
    fn safe_directory_relative_item_path_rejects_parent_segments() {
        assert!(safe_directory_relative_item_path("src", "src/../outside.rs").is_err());
    }

    #[test]
    fn safe_directory_relative_item_path_rejects_absolute_paths() {
        assert!(safe_directory_relative_item_path("", "/tmp/outside.rs").is_err());
    }

    #[test]
    fn safe_directory_relative_item_path_rejects_separator_aliases() {
        assert!(safe_directory_relative_item_path("src", "src\\nested\\lib.rs").is_err());
    }

    #[test]
    fn safe_directory_relative_item_path_rejects_empty_paths() {
        assert!(safe_directory_relative_item_path("", "").is_err());
    }

    #[test]
    fn safe_directory_relative_item_path_rejects_paths_outside_remote_root() {
        assert!(safe_directory_relative_item_path("src", "other/lib.rs").is_err());
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

    #[test]
    fn validated_metadata_file_name_accepts_safe_names() {
        assert_eq!(
            validated_metadata_file_name(Some("README.md"), "README.md").expect("safe"),
            "README.md"
        );
    }

    #[test]
    fn validated_metadata_file_name_rejects_traversal_and_falls_back() {
        assert_eq!(
            validated_metadata_file_name(Some("../../etc/passwd"), "docs/config.toml")
                .expect("fallback"),
            "config.toml"
        );
        assert_eq!(
            validated_metadata_file_name(Some(".."), "..").expect("fallback"),
            "downloaded-file"
        );
    }

    #[test]
    fn media_file_url_targets_media_host() {
        assert_eq!(
            build_media_file_url("onnx/models", "main", "validated/mnist-8.onnx"),
            "https://media.githubusercontent.com/media/onnx/models/main/validated/mnist-8.onnx"
        );
    }

    #[test]
    fn raw_file_url_encodes_repo_path_segments() {
        assert_eq!(
            build_raw_file_url("owner/repo", "main", "src/a b/c.rs"),
            "https://raw.githubusercontent.com/owner/repo/main/src/a%20b/c.rs"
        );
        assert_eq!(
            build_raw_file_url("/owner/repo/", "abc123", "/README.md"),
            "https://raw.githubusercontent.com/owner/repo/abc123/README.md"
        );
    }
}
