use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub(super) struct TreeResponse {
    #[serde(default)]
    pub(super) tree: Vec<TreeEntry>,
    #[serde(default)]
    pub(super) truncated: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub(super) struct TreeEntry {
    pub(super) path: String,
    #[serde(default)]
    pub(super) mode: String,
    #[serde(rename = "type", default)]
    pub(super) kind: String,
    #[serde(default)]
    pub(super) sha: String,
    #[serde(default)]
    #[allow(dead_code)]
    pub(super) size: Option<u64>,
}

impl TreeEntry {
    pub(super) fn is_tree(&self) -> bool {
        self.kind == "tree"
    }

    pub(super) fn is_symlink(&self) -> bool {
        self.kind == "blob" && self.mode == "120000"
    }

    pub(super) fn is_submodule(&self) -> bool {
        self.kind == "commit" || self.mode == "160000"
    }

    pub(super) fn is_regular_file(&self) -> bool {
        self.kind == "blob" && self.mode != "120000"
    }
}

pub(super) fn build_tree_url(api_base: &str, repo: &str, rev: &str, recursive: bool) -> String {
    let encoded_rev = urlencoding::encode(rev);
    let mut url = format!(
        "{}/repos/{}/git/trees/{}",
        api_base.trim_end_matches('/'),
        repo,
        encoded_rev
    );
    if recursive {
        url.push_str("?recursive=1");
    }
    url
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tree_url_encodes_revision_and_builds_recursive_query() {
        assert_eq!(
            build_tree_url("https://api.github.com/", "owner/repo", "feature/one", true),
            "https://api.github.com/repos/owner/repo/git/trees/feature%2Fone?recursive=1"
        );
        assert_eq!(
            build_tree_url("https://ghe.example/api/v3", "owner/repo", "abc123", false),
            "https://ghe.example/api/v3/repos/owner/repo/git/trees/abc123"
        );
    }

    #[test]
    fn tree_entry_classification_uses_mode_and_type() {
        let file = TreeEntry {
            path: "src/main.rs".to_string(),
            mode: "100644".to_string(),
            kind: "blob".to_string(),
            sha: "a".to_string(),
            size: Some(10),
        };
        let executable = TreeEntry {
            mode: "100755".to_string(),
            ..file.clone()
        };
        let symlink = TreeEntry {
            mode: "120000".to_string(),
            ..file.clone()
        };
        let submodule = TreeEntry {
            mode: "160000".to_string(),
            kind: "commit".to_string(),
            ..file.clone()
        };

        assert!(file.is_regular_file());
        assert!(executable.is_regular_file());
        assert!(symlink.is_symlink());
        assert!(!symlink.is_regular_file());
        assert!(submodule.is_submodule());
    }
}
