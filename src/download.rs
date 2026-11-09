mod models;
mod paths;
mod raw;
mod transport;
mod tree;

use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::thread;

use reqwest::blocking::Client;
use serde::Deserialize;
use serde_json::Value;

use crate::cli::{LfsMode, PrefixProxyMode, ResolvedOptions, SymlinkMode};
use crate::error::{AppError, FailedFile, PartialStats};
use crate::i18n::Language;
use crate::output::Output;

use self::models::ContentItem;
pub use self::paths::{
    build_contents_api_url, choose_directory_target, format_remote_path, join_proxy_url,
    normalize_repo_path, relative_item_path,
};
use self::paths::{
    build_media_file_url, build_raw_file_url, choose_file_target, file_name_from_remote_path,
    redact_url_for_display, safe_directory_relative_item_path,
};
use self::raw::{RawDownloadStrategy, should_attempt_prefix_proxy};
use self::transport::{build_client, fetch_bytes, send_json_request, stream_download};
use self::tree::{TreeEntry, TreeResponse, build_tree_url};

pub const DEFAULT_GH_PROXY: &str = "https://gh-proxy.com/";
pub const DEFAULT_GITHUB_API_BASE: &str = "https://api.github.com";

#[derive(Debug, Clone)]
pub struct RuntimeConfig {
    pub api_base: String,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            api_base: DEFAULT_GITHUB_API_BASE.to_string(),
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct DownloadStats {
    pub files_downloaded: usize,
    pub skipped_existing_files: usize,
    pub skipped_entries: usize,
    pub skipped_symlinks: usize,
    pub skipped_submodules: usize,
}

#[derive(Debug)]
pub struct RunOutcome {
    pub saved_path: PathBuf,
    pub stats: DownloadStats,
    pub ref_sha: Option<String>,
}

#[derive(Debug, Clone)]
struct DownloadJob {
    item: ContentItem,
    local_target: PathBuf,
    shown_path: String,
    rev: Option<String>,
    private_repo: bool,
    kind: DownloadJobKind,
}

#[derive(Debug, Clone)]
enum DownloadJobKind {
    File,
    Symlink { target: String },
}

#[derive(Debug, Clone, Deserialize)]
struct RepositoryInfo {
    #[serde(default)]
    default_branch: Option<String>,
    #[serde(rename = "private", default)]
    is_private: bool,
}

#[derive(Debug, Deserialize)]
struct CommitInfo {
    #[serde(default)]
    sha: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SaveFileOutcome {
    Downloaded,
    SkippedExisting,
}

pub struct Runner {
    config: RuntimeConfig,
    output: Output,
    repo_info: std::sync::Mutex<Option<RepositoryInfo>>,
    commit_sha: std::sync::Mutex<Option<String>>,
}

impl Runner {
    pub fn new(config: RuntimeConfig, output: Output) -> Self {
        Self {
            config,
            output,
            repo_info: std::sync::Mutex::new(None),
            commit_sha: std::sync::Mutex::new(None),
        }
    }

    pub fn run(&self, options: &ResolvedOptions) -> Result<RunOutcome, AppError> {
        let client = build_client()?;
        let private_repo = if options.token.is_some() {
            let info = self.load_repo_info(&client, options)?;
            if options.debug {
                self.output
                    .debug_line(&format!("[debug] repository-private: {}", info.is_private));
            }
            info.is_private
        } else {
            false
        };
        let commit_sha = self.resolve_commit_sha(&client, options)?;
        let mut stats = DownloadStats::default();
        let saved_path =
            self.detect_and_download(&client, options, &mut stats, private_repo, &commit_sha)?;
        self.output
            .completion(&options.repo, &options.remote_path, &saved_path, &stats);
        Ok(RunOutcome {
            saved_path,
            stats,
            ref_sha: Some(commit_sha),
        })
    }

    fn detect_and_download(
        &self,
        client: &Client,
        options: &ResolvedOptions,
        stats: &mut DownloadStats,
        private_repo: bool,
        commit_sha: &str,
    ) -> Result<PathBuf, AppError> {
        let normalized_path = normalize_repo_path(&options.remote_path);
        let metadata_url = build_contents_api_url(
            &self.config.api_base,
            &options.repo,
            &normalized_path,
            Some(commit_sha),
        );
        self.debug_log_url(options, "metadata-url", &metadata_url);
        let response = self.get_json(client, &metadata_url, options)?;

        if response.is_object() {
            let item: ContentItem =
                serde_json::from_value(response).map_err(|err| AppError::Json(err.to_string()))?;
            if let Some(kind) = item.kind.as_deref()
                && matches!(kind, "submodule" | "symlink")
            {
                return Err(AppError::NonRegularEntry {
                    path: normalized_path.clone(),
                    kind: kind.to_string(),
                });
            }
            let final_target = choose_file_target(
                &options.local_target,
                &normalized_path,
                &item,
                options.local_target_is_dir_hint,
            )?;
            let shown_path = item
                .name
                .clone()
                .unwrap_or_else(|| file_name_from_remote_path(&normalized_path));
            match self.save_file(
                client,
                options,
                &item,
                &final_target,
                &shown_path,
                Some(commit_sha),
                private_repo,
            )? {
                SaveFileOutcome::Downloaded => stats.files_downloaded += 1,
                SaveFileOutcome::SkippedExisting => stats.skipped_existing_files += 1,
            }
            return Ok(final_target);
        }

        if response.is_array() {
            let directory_target = choose_directory_target(&options.local_target, &normalized_path);
            let jobs = self.collect_directory_jobs(
                client,
                options,
                stats,
                &normalized_path,
                &directory_target,
                commit_sha,
                private_repo,
            )?;
            let worker_threads =
                effective_directory_worker_threads(jobs.len(), options.concurrency);
            self.output
                .found_directory(jobs.len(), &normalized_path, worker_threads);
            fs::create_dir_all(&directory_target).map_err(|source| AppError::Io {
                path: directory_target.clone(),
                source,
            })?;
            self.output.created_directory(&directory_target);
            self.download_directory_files(client, options, stats, jobs)?;
            return Ok(directory_target);
        }

        Err(AppError::UnexpectedApiResponse)
    }

    fn load_repo_info(
        &self,
        client: &Client,
        options: &ResolvedOptions,
    ) -> Result<RepositoryInfo, AppError> {
        if let Some(info) = self.repo_info.lock().expect("repository info lock").clone() {
            return Ok(info);
        }

        let url = format!(
            "{}/repos/{}",
            self.config.api_base.trim_end_matches('/'),
            options.repo
        );
        self.debug_log_url(options, "repository-url", &url);
        let response = self.get_json(client, &url, options)?;
        let info: RepositoryInfo =
            serde_json::from_value(response).map_err(|err| AppError::Json(err.to_string()))?;
        *self.repo_info.lock().expect("repository info lock") = Some(info.clone());
        Ok(info)
    }

    fn resolve_commit_sha(
        &self,
        client: &Client,
        options: &ResolvedOptions,
    ) -> Result<String, AppError> {
        if let Some(sha) = self.commit_sha.lock().expect("commit sha lock").clone() {
            return Ok(sha);
        }

        let requested_ref = options
            .git_ref
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty());

        let resolved_ref = match requested_ref {
            Some(git_ref) => git_ref.to_string(),
            None => {
                let info = self.load_repo_info(client, options)?;
                let branch = info.default_branch.ok_or(AppError::UnexpectedApiResponse)?;
                if options.debug {
                    self.output
                        .debug_line(&format!("[debug] default-branch: {}", branch));
                }
                branch
            }
        };

        let sha = if is_full_commit_sha(&resolved_ref) {
            resolved_ref
        } else {
            let url = format!(
                "{}/repos/{}/commits/{}",
                self.config.api_base.trim_end_matches('/'),
                options.repo,
                urlencoding::encode(&resolved_ref)
            );
            self.debug_log_url(options, "commit-url", &url);
            let response = self.get_json(client, &url, options)?;
            let commit: CommitInfo =
                serde_json::from_value(response).map_err(|err| AppError::Json(err.to_string()))?;
            commit.sha.ok_or_else(|| {
                AppError::InvalidPath(format!(
                    "the requested ref {:?} did not resolve to a commit",
                    resolved_ref
                ))
            })?
        };

        if options.debug {
            self.output
                .debug_line(&format!("[debug] commit-sha: {}", sha));
        }
        *self.commit_sha.lock().expect("commit sha lock") = Some(sha.clone());
        Ok(sha)
    }

    fn collect_directory_jobs(
        &self,
        client: &Client,
        options: &ResolvedOptions,
        stats: &mut DownloadStats,
        remote_root: &str,
        directory_target: &Path,
        rev: &str,
        private_repo: bool,
    ) -> Result<Vec<DownloadJob>, AppError> {
        let remote_root_normalized = normalize_repo_path(remote_root);
        let (tree_rev, tree_prefix) =
            match self.resolve_subtree_sha(client, options, rev, &remote_root_normalized)? {
                Some(tree_sha) => (tree_sha, format!("{}/", remote_root_normalized)),
                None => (rev.to_string(), String::new()),
            };
        let entries = self.fetch_tree_entries(client, options, &tree_rev, &tree_prefix)?;
        let mut jobs = Vec::new();

        for entry in entries {
            if entry.is_tree() {
                continue;
            }
            if !entry_is_within_root(remote_root, &entry.path) {
                continue;
            }

            if entry.is_symlink() {
                self.collect_symlink_job(
                    client,
                    options,
                    stats,
                    &entry,
                    remote_root,
                    directory_target,
                    rev,
                    private_repo,
                    &mut jobs,
                )?;
                continue;
            }

            if entry.is_submodule() {
                stats.skipped_submodules += 1;
                self.warn_submodule(client, options, &entry, rev)?;
                continue;
            }

            if !entry.is_regular_file() {
                stats.skipped_entries += 1;
                self.warn_skipped_tree_entry(options, &entry);
                continue;
            }

            let relative_part = safe_directory_relative_item_path(remote_root, &entry.path)?;
            let name = Path::new(&entry.path)
                .file_name()
                .and_then(|value| value.to_str())
                .map(ToOwned::to_owned);
            jobs.push(DownloadJob {
                item: ContentItem {
                    name,
                    path: Some(entry.path.clone()),
                    kind: Some("file".to_string()),
                    download_url: self.raw_url_for(options, rev, &entry.path),
                },
                local_target: directory_target.join(&relative_part),
                shown_path: relative_part,
                rev: Some(rev.to_string()),
                private_repo,
                kind: DownloadJobKind::File,
            });
        }

        Ok(jobs)
    }

    #[allow(clippy::too_many_arguments)]
    fn collect_symlink_job(
        &self,
        client: &Client,
        options: &ResolvedOptions,
        stats: &mut DownloadStats,
        entry: &TreeEntry,
        remote_root: &str,
        directory_target: &Path,
        rev: &str,
        private_repo: bool,
        jobs: &mut Vec<DownloadJob>,
    ) -> Result<(), AppError> {
        let relative_part = safe_directory_relative_item_path(remote_root, &entry.path)?;

        match options.symlinks {
            SymlinkMode::Skip => {
                stats.skipped_symlinks += 1;
                self.warn_skipped_symlink(options, &entry.path, None);
                Ok(())
            }
            SymlinkMode::Link => {
                let target = self.fetch_symlink_target(client, options, &entry.sha)?;
                jobs.push(DownloadJob {
                    item: ContentItem {
                        name: Path::new(&entry.path)
                            .file_name()
                            .and_then(|value| value.to_str())
                            .map(ToOwned::to_owned),
                        path: Some(entry.path.clone()),
                        kind: Some("symlink".to_string()),
                        download_url: None,
                    },
                    local_target: directory_target.join(&relative_part),
                    shown_path: relative_part,
                    rev: Some(rev.to_string()),
                    private_repo,
                    kind: DownloadJobKind::Symlink { target },
                });
                Ok(())
            }
            SymlinkMode::Auto => {
                let resolved = self.resolve_symlink_kind(client, options, &entry.path, rev)?;
                if resolved.as_deref() == Some("file") {
                    let name = Path::new(&entry.path)
                        .file_name()
                        .and_then(|value| value.to_str())
                        .map(ToOwned::to_owned);
                    jobs.push(DownloadJob {
                        item: ContentItem {
                            name,
                            path: Some(entry.path.clone()),
                            kind: Some("file".to_string()),
                            download_url: None,
                        },
                        local_target: directory_target.join(&relative_part),
                        shown_path: relative_part,
                        rev: Some(rev.to_string()),
                        private_repo,
                        kind: DownloadJobKind::File,
                    });
                } else {
                    stats.skipped_symlinks += 1;
                    self.warn_skipped_symlink(options, &entry.path, None);
                }
                Ok(())
            }
        }
    }

    fn resolve_symlink_kind(
        &self,
        client: &Client,
        options: &ResolvedOptions,
        repo_path: &str,
        rev: &str,
    ) -> Result<Option<String>, AppError> {
        let url =
            build_contents_api_url(&self.config.api_base, &options.repo, repo_path, Some(rev));
        self.debug_log_url(options, "symlink-url", &url);
        let response = self.get_json(client, &url, options)?;
        Ok(response
            .get("type")
            .and_then(Value::as_str)
            .map(str::to_string))
    }

    fn fetch_symlink_target(
        &self,
        client: &Client,
        options: &ResolvedOptions,
        blob_sha: &str,
    ) -> Result<String, AppError> {
        if blob_sha.trim().is_empty() {
            return Err(AppError::UnexpectedApiResponse);
        }
        let url = format!(
            "{}/repos/{}/git/blobs/{}",
            self.config.api_base.trim_end_matches('/'),
            options.repo,
            blob_sha
        );
        self.debug_log_url(options, "symlink-blob-url", &url);
        let bytes = fetch_bytes(
            client,
            &url,
            Some("application/vnd.github.raw"),
            options.token.as_deref(),
            4096,
        )?;
        let target = String::from_utf8_lossy(&bytes).trim().to_string();
        if target.is_empty() {
            return Err(AppError::UnexpectedApiResponse);
        }
        Ok(target)
    }

    fn warn_skipped_tree_entry(&self, options: &ResolvedOptions, entry: &TreeEntry) {
        self.output.warning(&format!(
            "{} {} ({})",
            match options.language {
                Language::Zh => "跳过不支持的条目：",
                Language::En => "Skipping unsupported entry:",
            },
            entry.path,
            entry.kind
        ));
    }

    fn warn_skipped_symlink(&self, options: &ResolvedOptions, path: &str, detail: Option<&str>) {
        let message = match options.language {
            Language::Zh => match detail {
                Some(detail) => format!("跳过符号链接：{}（{}）", path, detail),
                None => format!("跳过符号链接：{}", path),
            },
            Language::En => match detail {
                Some(detail) => format!("Skipping symlink: {} ({})", path, detail),
                None => format!("Skipping symlink: {}", path),
            },
        };
        self.output.warning(&message);
    }

    fn warn_submodule(
        &self,
        client: &Client,
        options: &ResolvedOptions,
        entry: &TreeEntry,
        rev: &str,
    ) -> Result<(), AppError> {
        let url =
            build_contents_api_url(&self.config.api_base, &options.repo, &entry.path, Some(rev));
        let submodule_url = match self.get_json(client, &url, options) {
            Ok(response) => response
                .get("submodule_git_url")
                .and_then(Value::as_str)
                .unwrap_or("unknown")
                .to_string(),
            Err(_) => "unknown".to_string(),
        };
        self.output.warning(&match options.language {
            Language::Zh => format!(
                "跳过子模块：{}（{}，commit {}）",
                entry.path, submodule_url, entry.sha
            ),
            Language::En => format!(
                "Skipping submodule: {} ({}, commit {})",
                entry.path, submodule_url, entry.sha
            ),
        });
        Ok(())
    }

    fn raw_url_for(&self, options: &ResolvedOptions, rev: &str, repo_path: &str) -> Option<String> {
        if self.is_default_github_api() {
            Some(build_raw_file_url(&options.repo, rev, repo_path))
        } else {
            None
        }
    }

    fn is_default_github_api(&self) -> bool {
        self.config.api_base.trim_end_matches('/') == DEFAULT_GITHUB_API_BASE
    }

    fn resolve_subtree_sha(
        &self,
        client: &Client,
        options: &ResolvedOptions,
        commit_sha: &str,
        remote_root: &str,
    ) -> Result<Option<String>, AppError> {
        if remote_root.is_empty() {
            return Ok(None);
        }

        let mut current_sha = commit_sha.to_string();
        for segment in remote_root.split('/').filter(|value| !value.is_empty()) {
            let url = build_tree_url(&self.config.api_base, &options.repo, &current_sha, false);
            self.debug_log_url(options, "tree-lookup-url", &url);
            let response = self.get_json(client, &url, options)?;
            let tree: TreeResponse =
                serde_json::from_value(response).map_err(|err| AppError::Json(err.to_string()))?;
            let entry = tree
                .tree
                .iter()
                .find(|entry| entry.is_tree() && entry.path == segment)
                .ok_or(AppError::UnexpectedApiResponse)?;
            current_sha = entry.sha.clone();
        }

        Ok(Some(current_sha))
    }

    fn fetch_tree_payload(
        &self,
        client: &Client,
        options: &ResolvedOptions,
        url: &str,
    ) -> Result<Option<TreeResponse>, AppError> {
        const MAX_TREE_RESPONSE_BYTES: usize = 16 * 1024 * 1024;
        let bytes = fetch_bytes(
            client,
            url,
            Some("application/vnd.github+json"),
            options.token.as_deref(),
            MAX_TREE_RESPONSE_BYTES,
        )?;
        Ok(serde_json::from_slice::<TreeResponse>(&bytes).ok())
    }

    fn fetch_tree_entries(
        &self,
        client: &Client,
        options: &ResolvedOptions,
        rev: &str,
        prefix: &str,
    ) -> Result<Vec<TreeEntry>, AppError> {
        let mut entries = Vec::new();
        self.fetch_tree_into(client, options, rev, prefix, &mut entries, 0)?;
        Ok(entries)
    }

    fn fetch_tree_into(
        &self,
        client: &Client,
        options: &ResolvedOptions,
        rev: &str,
        prefix: &str,
        entries: &mut Vec<TreeEntry>,
        depth: usize,
    ) -> Result<(), AppError> {
        const MAX_TREE_DEPTH: usize = 64;
        if depth > MAX_TREE_DEPTH {
            return Err(AppError::Json(
                "tree enumeration exceeded the maximum depth".to_string(),
            ));
        }

        let url = build_tree_url(&self.config.api_base, &options.repo, rev, true);
        self.debug_log_url(options, "tree-url", &url);
        let recursive = self.fetch_tree_payload(client, options, &url)?;

        if let Some(tree) = recursive
            && !tree.truncated
        {
            if options.debug {
                self.output
                    .debug_line("[debug] enumeration-mode: recursive");
            }
            entries.extend(tree.tree.into_iter().map(|mut entry| {
                entry.path = format!("{}{}", prefix, entry.path);
                entry
            }));
            return Ok(());
        }

        if options.debug {
            self.output
                .debug_line("[debug] enumeration-mode: non-recursive-fallback");
        }

        let url = build_tree_url(&self.config.api_base, &options.repo, rev, false);
        self.debug_log_url(options, "tree-url", &url);
        let root = self
            .fetch_tree_payload(client, options, &url)?
            .ok_or(AppError::UnexpectedApiResponse)?;

        for mut entry in root.tree {
            entry.path = format!("{}{}", prefix, entry.path);
            if entry.is_tree() {
                let child_prefix = format!("{}/", entry.path);
                self.fetch_tree_into(
                    client,
                    options,
                    &entry.sha,
                    &child_prefix,
                    entries,
                    depth + 1,
                )?;
            } else {
                entries.push(entry);
            }
        }

        Ok(())
    }

    fn save_file(
        &self,
        client: &Client,
        options: &ResolvedOptions,
        item: &ContentItem,
        local_target: &Path,
        shown_path: &str,
        rev: Option<&str>,
        private_repo: bool,
    ) -> Result<SaveFileOutcome, AppError> {
        match existing_target_kind(local_target)? {
            Some(ExistingTargetKind::Directory) => {
                return Err(AppError::TargetIsDirectory {
                    path: local_target.to_path_buf(),
                });
            }
            Some(ExistingTargetKind::Entry) if !options.overwrite => {
                self.output.skipping_existing(shown_path);
                return Ok(SaveFileOutcome::SkippedExisting);
            }
            _ => {}
        }

        let item_path = item
            .path
            .as_deref()
            .ok_or(AppError::MissingRepositoryPath)?;

        self.output.downloading(shown_path);

        let api_raw_primary = options.token.is_some() && private_repo;
        let mut attempts = Vec::new();
        let mut raw_error = None;

        if !api_raw_primary && let Some(download_url) = item.download_url.as_deref() {
            self.debug_log_url(options, "download-url", download_url);
            match self.download_from_raw_url(
                client,
                options,
                local_target,
                download_url,
                &mut attempts,
            ) {
                Ok(()) => {
                    self.finish_download(
                        client,
                        options,
                        item_path,
                        local_target,
                        rev,
                        private_repo,
                    )?;
                    return Ok(SaveFileOutcome::Downloaded);
                }
                Err(error) => {
                    self.output.warning(match options.language {
                        Language::Zh => "文件下载失败，正在改用 GitHub Raw API...",
                        Language::En => "File download failed, falling back to GitHub Raw API...",
                    });
                    raw_error = Some(error);
                }
            }
        }

        self.debug_log_strategy(options, RawDownloadStrategy::RawApi);
        match self.stream_contents_api_file(client, options, item_path, local_target, rev) {
            Ok(()) => {
                self.finish_download(client, options, item_path, local_target, rev, private_repo)?;
                Ok(SaveFileOutcome::Downloaded)
            }
            Err(api_error) => {
                if api_raw_primary {
                    attempts.push(format!("raw api: {}", summarize_error(&api_error)));
                    return Err(AppError::Download {
                        remote_path: item_path.to_string(),
                        local_path: local_target.to_path_buf(),
                        attempts,
                        cause: Box::new(api_error),
                    });
                }
                if attempts.is_empty() && raw_error.is_none() {
                    return Err(api_error);
                }

                attempts.push(format!("raw api: {}", summarize_error(&api_error)));
                Err(AppError::Download {
                    remote_path: item_path.to_string(),
                    local_path: local_target.to_path_buf(),
                    attempts,
                    cause: Box::new(raw_error.unwrap_or(api_error)),
                })
            }
        }
    }

    fn download_directory_files(
        &self,
        client: &Client,
        options: &ResolvedOptions,
        stats: &mut DownloadStats,
        jobs: Vec<DownloadJob>,
    ) -> Result<(), AppError> {
        let worker = |job| self.download_job(client, options, job);
        let outcome = run_bounded_work(&jobs, options.concurrency, options.keep_going, &worker);

        for result in outcome.results {
            match result {
                SaveFileOutcome::Downloaded => stats.files_downloaded += 1,
                SaveFileOutcome::SkippedExisting => stats.skipped_existing_files += 1,
            }
        }

        if outcome.errors.is_empty() {
            return Ok(());
        }

        let failures: Vec<FailedFile> = outcome.errors.iter().map(failed_file_from_error).collect();
        Err(AppError::DirectoryTransfer {
            failed_count: failures.len(),
            failures,
            stats: PartialStats {
                downloaded: stats.files_downloaded,
                skipped_existing: stats.skipped_existing_files,
                skipped_entries: stats.skipped_entries,
            },
            aborted_early: outcome.aborted_early,
        })
    }

    fn download_job(
        &self,
        client: &Client,
        options: &ResolvedOptions,
        job: DownloadJob,
    ) -> Result<SaveFileOutcome, AppError> {
        if let DownloadJobKind::Symlink { target } = &job.kind {
            return self.save_symlink(options, &job, target);
        }

        match self.save_file(
            client,
            options,
            &job.item,
            &job.local_target,
            &job.shown_path,
            job.rev.as_deref(),
            job.private_repo,
        ) {
            Ok(outcome) => Ok(outcome),
            Err(error) => {
                if matches!(error, AppError::Download { .. }) {
                    return Err(error);
                }
                Err(AppError::Download {
                    remote_path: job
                        .item
                        .path
                        .clone()
                        .unwrap_or_else(|| job.shown_path.clone()),
                    local_path: job.local_target.clone(),
                    attempts: Vec::new(),
                    cause: Box::new(error),
                })
            }
        }
    }

    fn save_symlink(
        &self,
        options: &ResolvedOptions,
        job: &DownloadJob,
        target: &str,
    ) -> Result<SaveFileOutcome, AppError> {
        match existing_target_kind(&job.local_target)? {
            Some(ExistingTargetKind::Directory) => {
                return Err(AppError::TargetIsDirectory {
                    path: job.local_target.clone(),
                });
            }
            Some(ExistingTargetKind::Entry) if !options.overwrite => {
                self.output.skipping_existing(&job.shown_path);
                return Ok(SaveFileOutcome::SkippedExisting);
            }
            _ => {}
        }

        if let Some(parent) = job.local_target.parent() {
            fs::create_dir_all(parent).map_err(|source| AppError::Io {
                path: parent.to_path_buf(),
                source,
            })?;
        }

        #[cfg(unix)]
        {
            if matches!(
                existing_target_kind(&job.local_target)?,
                Some(ExistingTargetKind::Entry)
            ) {
                fs::remove_file(&job.local_target).map_err(|source| AppError::Io {
                    path: job.local_target.clone(),
                    source,
                })?;
            }
            self.output.downloading(&job.shown_path);
            std::os::unix::fs::symlink(target, &job.local_target).map_err(|source| {
                AppError::Io {
                    path: job.local_target.clone(),
                    source,
                }
            })?;
            Ok(SaveFileOutcome::Downloaded)
        }

        #[cfg(not(unix))]
        {
            let _ = (options, target);
            Err(AppError::InvalidPath(
                "--symlinks link is not supported on this platform".to_string(),
            ))
        }
    }

    fn finish_download(
        &self,
        client: &Client,
        options: &ResolvedOptions,
        remote_path: &str,
        local_target: &Path,
        rev: Option<&str>,
        private_repo: bool,
    ) -> Result<(), AppError> {
        let Some(pointer_size) = detect_lfs_pointer(local_target)? else {
            return Ok(());
        };

        match options.lfs {
            LfsMode::Error => {
                let _ = fs::remove_file(local_target);
                Err(AppError::LfsObject {
                    remote_path: remote_path.to_string(),
                    local_path: local_target.to_path_buf(),
                })
            }
            LfsMode::Media => {
                let unsupported = || AppError::LfsObject {
                    remote_path: remote_path.to_string(),
                    local_path: local_target.to_path_buf(),
                };
                if !self.is_default_github_api() {
                    let _ = fs::remove_file(local_target);
                    return Err(unsupported());
                }
                let Some(rev) = rev else {
                    let _ = fs::remove_file(local_target);
                    return Err(unsupported());
                };
                let media_url = build_media_file_url(&options.repo, rev, remote_path);
                self.debug_log_url(options, "lfs-media-url", &media_url);
                let token = if private_repo {
                    options.token.as_deref()
                } else {
                    None
                };
                let mut last_error = None;
                for _attempt in 0..2 {
                    let _ = fs::remove_file(local_target);
                    match self.stream_download(client, &media_url, local_target, None, token) {
                        Ok(()) => {
                            if detect_lfs_pointer(local_target)?.is_some() {
                                let _ = fs::remove_file(local_target);
                                last_error = Some(unsupported());
                                continue;
                            }
                            last_error = None;
                            break;
                        }
                        Err(error) => {
                            let _ = fs::remove_file(local_target);
                            last_error = Some(error);
                        }
                    }
                }
                if let Some(error) = last_error {
                    return Err(error);
                }
                if options.debug {
                    self.output
                        .debug_line(&format!("[debug] lfs-pointer-bytes: {}", pointer_size));
                }
                Ok(())
            }
        }
    }

    fn stream_contents_api_file(
        &self,
        client: &Client,
        options: &ResolvedOptions,
        remote_path: &str,
        local_target: &Path,
        rev: Option<&str>,
    ) -> Result<(), AppError> {
        let api_url =
            build_contents_api_url(&self.config.api_base, &options.repo, remote_path, rev);
        self.debug_log_url(options, "raw-api-url", &api_url);

        self.stream_download(
            client,
            &api_url,
            local_target,
            Some("application/vnd.github.raw"),
            options.token.as_deref(),
        )
    }

    fn should_attempt_prefix_proxy(&self, options: &ResolvedOptions, error: &AppError) -> bool {
        should_attempt_prefix_proxy(options.prefix_mode, &options.proxy_base, error)
    }

    fn download_from_raw_url(
        &self,
        client: &Client,
        options: &ResolvedOptions,
        local_target: &Path,
        download_url: &str,
        attempts: &mut Vec<String>,
    ) -> Result<(), AppError> {
        match options.prefix_mode {
            PrefixProxyMode::Direct => {
                self.debug_log_strategy(options, RawDownloadStrategy::DirectUrl);
                let result = self.stream_download(client, download_url, local_target, None, None);
                record_attempt(attempts, "direct raw", &result);
                result
            }
            PrefixProxyMode::Fallback => {
                self.debug_log_strategy(options, RawDownloadStrategy::DirectUrl);
                let direct = self.stream_download(client, download_url, local_target, None, None);
                record_attempt(attempts, "direct raw", &direct);
                match direct {
                    Ok(()) => Ok(()),
                    Err(error) if self.should_attempt_prefix_proxy(options, &error) => {
                        let proxied_url = join_proxy_url(&options.proxy_base, download_url);
                        self.debug_log_strategy(options, RawDownloadStrategy::PrefixProxy);
                        self.debug_log_url(options, "prefix-url", &proxied_url);
                        self.output.warning(&match options.language {
                            Language::Zh => format!(
                                "直连文件下载失败，正在通过前缀代理重试：{}",
                                redact_url_for_display(&proxied_url)
                            ),
                            Language::En => format!(
                                "Direct file download failed, retrying through prefix proxy: {}",
                                redact_url_for_display(&proxied_url)
                            ),
                        });
                        let result =
                            self.stream_download(client, &proxied_url, local_target, None, None);
                        record_attempt(attempts, "prefix proxy", &result);
                        result
                    }
                    Err(error) => Err(error),
                }
            }
            PrefixProxyMode::Prefer => {
                if options.proxy_base.trim().is_empty() {
                    self.debug_log_strategy(options, RawDownloadStrategy::DirectUrl);
                    let result =
                        self.stream_download(client, download_url, local_target, None, None);
                    record_attempt(attempts, "direct raw", &result);
                    return result;
                }

                let proxied_url = join_proxy_url(&options.proxy_base, download_url);
                self.debug_log_strategy(options, RawDownloadStrategy::PrefixProxy);
                self.debug_log_url(options, "prefix-url", &proxied_url);
                let proxied = self.stream_download(client, &proxied_url, local_target, None, None);
                record_attempt(attempts, "prefix proxy", &proxied);
                match proxied {
                    Ok(()) => Ok(()),
                    Err(_) => {
                        self.output.warning(match options.language {
                            Language::Zh => "前缀代理下载失败，正在回退直连文件 URL...",
                            Language::En => {
                                "Prefix proxy download failed, falling back to direct file URL..."
                            }
                        });
                        self.debug_log_strategy(options, RawDownloadStrategy::DirectUrl);
                        let result =
                            self.stream_download(client, download_url, local_target, None, None);
                        record_attempt(attempts, "direct raw", &result);
                        result
                    }
                }
            }
        }
    }

    fn debug_log_url(&self, options: &ResolvedOptions, label: &str, url: &str) {
        if !options.debug {
            return;
        }

        self.output.debug_line(&format!(
            "[debug] {}: {}",
            label,
            redact_url_for_display(url)
        ));
    }

    fn debug_log_strategy(&self, options: &ResolvedOptions, strategy: RawDownloadStrategy) {
        if !options.debug {
            return;
        }

        self.output.debug_line(&format!(
            "[debug] raw-download-strategy: {}",
            strategy.as_str()
        ));
    }

    fn get_json(
        &self,
        client: &Client,
        url: &str,
        options: &ResolvedOptions,
    ) -> Result<Value, AppError> {
        self.send_json_request(
            client,
            url,
            Some("application/vnd.github+json"),
            options.token.as_deref(),
        )
    }

    fn send_json_request(
        &self,
        client: &Client,
        url: &str,
        accept: Option<&str>,
        token: Option<&str>,
    ) -> Result<Value, AppError> {
        send_json_request(client, url, accept, token)
    }

    fn stream_download(
        &self,
        client: &Client,
        url: &str,
        local_target: &Path,
        accept: Option<&str>,
        token: Option<&str>,
    ) -> Result<(), AppError> {
        stream_download(client, url, local_target, accept, token)
    }
}

struct WorkOutcome<R, E> {
    results: Vec<R>,
    errors: Vec<E>,
    aborted_early: bool,
}

fn run_bounded_work<T, R, E, F>(
    items: &[T],
    concurrency: usize,
    keep_going: bool,
    worker: &F,
) -> WorkOutcome<R, E>
where
    T: Clone + Send,
    R: Send,
    E: Send,
    F: Fn(T) -> Result<R, E> + Sync,
{
    let mut results = Vec::new();
    let mut errors = Vec::new();

    if items.is_empty() {
        return WorkOutcome {
            results,
            errors,
            aborted_early: false,
        };
    }

    let concurrency = concurrency.max(1);

    for batch in items.chunks(concurrency) {
        let mut batch_results = Vec::with_capacity(batch.len());
        let mut batch_errors = Vec::new();

        thread::scope(|scope| {
            let mut handles = Vec::with_capacity(batch.len());
            for item in batch.iter().cloned() {
                handles.push(scope.spawn(move || worker(item)));
            }

            for handle in handles {
                match handle.join().expect("worker thread panicked") {
                    Ok(result) => batch_results.push(result),
                    Err(error) => batch_errors.push(error),
                }
            }
        });

        results.extend(batch_results);
        let had_errors = !batch_errors.is_empty();
        errors.extend(batch_errors);

        if had_errors && !keep_going {
            return WorkOutcome {
                results,
                errors,
                aborted_early: true,
            };
        }
    }

    WorkOutcome {
        results,
        errors,
        aborted_early: false,
    }
}

fn entry_is_within_root(remote_root: &str, entry_path: &str) -> bool {
    let root = normalize_repo_path(remote_root);
    if root.is_empty() {
        return true;
    }
    entry_path == root || entry_path.starts_with(&format!("{}/", root))
}

fn is_full_commit_sha(value: &str) -> bool {
    value.len() == 40 && value.chars().all(|character| character.is_ascii_hexdigit())
}

fn effective_directory_worker_threads(file_count: usize, concurrency: usize) -> usize {
    file_count.min(concurrency.max(1))
}

fn detect_lfs_pointer(path: &Path) -> Result<Option<usize>, AppError> {
    let mut file = fs::File::open(path).map_err(|source| AppError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    let mut buffer = [0u8; 1024];
    let read = file.read(&mut buffer).map_err(|source| AppError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    let prefix = &buffer[..read];
    let looks_like_pointer = prefix.starts_with(b"version https://git-lfs.github.com/spec/v1")
        && prefix.windows(10).any(|window| window == b"oid sha256");
    Ok(looks_like_pointer.then_some(read))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExistingTargetKind {
    Entry,
    Directory,
}

fn existing_target_kind(path: &Path) -> Result<Option<ExistingTargetKind>, AppError> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.is_dir() => Ok(Some(ExistingTargetKind::Directory)),
        Ok(_) => Ok(Some(ExistingTargetKind::Entry)),
        Err(source) if source.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(source) => Err(AppError::Io {
            path: path.to_path_buf(),
            source,
        }),
    }
}

fn record_attempt(attempts: &mut Vec<String>, label: &str, result: &Result<(), AppError>) {
    if let Err(error) = result {
        attempts.push(format!("{}: {}", label, summarize_error(error)));
    }
}

fn failed_file_from_error(error: &AppError) -> FailedFile {
    match error {
        AppError::Download {
            remote_path,
            local_path,
            cause,
            ..
        } => FailedFile {
            path: remote_path.clone(),
            local_path: local_path.display().to_string(),
            reason: summarize_error(cause),
        },
        other => FailedFile {
            path: String::new(),
            local_path: String::new(),
            reason: summarize_error(other),
        },
    }
}

fn summarize_error(error: &AppError) -> String {
    match error {
        AppError::HttpStatus { status, .. } => format!("HTTP {}", status),
        AppError::Request { message, .. } => format!("network error: {}", message),
        AppError::Transfer { message, .. } => format!("network error: {}", message),
        AppError::InvalidUrl(_) => "unsupported URL".to_string(),
        other => other.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::Duration;

    use mockito::{Matcher, Server};
    use tempfile::tempdir;

    use super::*;

    const TEST_SHA: &str = "0123456789abcdef0123456789abcdef01234567";
    fn test_options(api_base: String, remote_path: &str, local_target: PathBuf) -> ResolvedOptions {
        ResolvedOptions {
            repo: "owner/repo".to_string(),
            remote_path: remote_path.to_string(),
            local_target,
            local_target_is_dir_hint: false,
            git_ref: Some(TEST_SHA.to_string()),
            token: None,
            api_base,
            proxy_base: DEFAULT_GH_PROXY.to_string(),
            prefix_mode: PrefixProxyMode::Fallback,
            symlinks: SymlinkMode::Auto,
            lfs: LfsMode::Error,
            concurrency: 4,
            language: Language::En,
            overwrite: false,
            keep_going: false,
            json: false,
            debug: false,
            no_color: true,
        }
    }

    fn test_runner(api_base: &str) -> Runner {
        Runner::new(
            RuntimeConfig {
                api_base: api_base.to_string(),
            },
            Output::new(false, Language::En),
        )
    }

    #[test]
    fn downloads_single_file() {
        let mut server = Server::new();
        let _root_tree = mock_root_tree(&mut server);
        let download_url = format!("{}/downloads/README.md", server.url());

        let _metadata = server
            .mock("GET", "/repos/owner/repo/contents/README.md")
            .match_query(Matcher::Any)
            .match_header("accept", "application/vnd.github+json")
            .with_status(200)
            .with_body(format!(
                r#"{{"name":"README.md","path":"README.md","type":"file","download_url":"{}"}}"#,
                download_url
            ))
            .create();

        let _download = server
            .mock("GET", "/downloads/README.md")
            .with_status(200)
            .with_body("hello from rust")
            .create();

        let dir = tempdir().expect("tempdir");
        let target = dir.path().join("README.md");
        let options = ResolvedOptions {
            repo: "owner/repo".to_string(),
            remote_path: "README.md".to_string(),
            local_target: target.clone(),
            local_target_is_dir_hint: false,
            git_ref: Some(TEST_SHA.to_string()),
            token: None,
            api_base: server.url(),
            proxy_base: DEFAULT_GH_PROXY.to_string(),
            prefix_mode: PrefixProxyMode::Fallback,
            symlinks: SymlinkMode::Auto,
            lfs: LfsMode::Error,
            concurrency: 4,
            language: Language::En,
            overwrite: false,
            keep_going: false,
            json: false,
            debug: false,
            no_color: true,
        };
        let runner = Runner::new(
            RuntimeConfig {
                api_base: server.url(),
            },
            Output::new(false, Language::En),
        );

        let outcome = runner.run(&options).expect("download should succeed");
        assert_eq!(outcome.saved_path, target);
        assert_eq!(outcome.ref_sha.as_deref(), Some(TEST_SHA));
        assert_eq!(fs::read_to_string(target).expect("file"), "hello from rust");
    }

    #[test]
    fn downloads_nested_directory() {
        let mut server = Server::new();
        let _root_tree = mock_root_tree(&mut server);

        let _listing = server
            .mock("GET", "/repos/owner/repo/contents/src")
            .match_query(Matcher::Any)
            .match_header("accept", "application/vnd.github+json")
            .with_status(200)
            .with_body("[]")
            .create();

        let _repo = server
            .mock("GET", "/repos/owner/repo")
            .with_status(200)
            .with_body(r#"{"default_branch":"main","private":false}"#)
            .create();

        let _tree = server
            .mock("GET", format!("/repos/owner/repo/git/trees/subtree-sha?recursive=1").as_str())
            .with_status(200)
            .with_body(
                r#"{"sha":"tree-sha","truncated":false,"tree":[{"path":"main.rs","mode":"100644","type":"blob","sha":"a","size":12},{"path":"nested/lib.rs","mode":"100644","type":"blob","sha":"b","size":18}]}"#,
            )
            .create();

        let _top_file = server
            .mock(
                "GET",
                format!("/repos/owner/repo/contents/src/main.rs?ref={TEST_SHA}").as_str(),
            )
            .match_header("accept", "application/vnd.github.raw")
            .with_status(200)
            .with_body("fn main() {}")
            .create();

        let _nested_file = server
            .mock(
                "GET",
                format!("/repos/owner/repo/contents/src/nested/lib.rs?ref={TEST_SHA}").as_str(),
            )
            .match_header("accept", "application/vnd.github.raw")
            .with_status(200)
            .with_body("pub fn helper() {}")
            .create();

        let dir = tempdir().expect("tempdir");
        let options = ResolvedOptions {
            repo: "owner/repo".to_string(),
            remote_path: "src".to_string(),
            local_target: dir.path().join("downloads"),
            local_target_is_dir_hint: false,
            git_ref: Some(TEST_SHA.to_string()),
            token: None,
            api_base: server.url(),
            proxy_base: DEFAULT_GH_PROXY.to_string(),
            prefix_mode: PrefixProxyMode::Fallback,
            symlinks: SymlinkMode::Auto,
            lfs: LfsMode::Error,
            concurrency: 4,
            language: Language::En,
            overwrite: false,
            keep_going: false,
            json: false,
            debug: false,
            no_color: true,
        };
        let runner = Runner::new(
            RuntimeConfig {
                api_base: server.url(),
            },
            Output::new(false, Language::En),
        );

        let outcome = runner.run(&options).expect("download should succeed");
        assert_eq!(outcome.stats.files_downloaded, 2);
        assert_eq!(
            fs::read_to_string(outcome.saved_path.join("main.rs")).expect("main"),
            "fn main() {}"
        );
        assert_eq!(
            fs::read_to_string(outcome.saved_path.join("nested/lib.rs")).expect("nested"),
            "pub fn helper() {}"
        );
    }

    #[test]
    fn directory_download_resolves_default_branch_to_commit() {
        let mut server = Server::new();
        let _root_tree = mock_root_tree(&mut server);
        let _repo = server
            .mock("GET", "/repos/owner/repo")
            .with_status(200)
            .with_body(r#"{"default_branch":"main","private":false}"#)
            .create();
        let _commit = server
            .mock("GET", "/repos/owner/repo/commits/main")
            .with_status(200)
            .with_body(format!(r#"{{"sha":"{TEST_SHA}"}}"#).as_str())
            .create();
        let _listing = server
            .mock("GET", "/repos/owner/repo/contents/src")
            .match_query(Matcher::Any)
            .with_status(200)
            .with_body("[]")
            .create();
        let _tree = server
            .mock(
                "GET",
                format!("/repos/owner/repo/git/trees/subtree-sha?recursive=1").as_str(),
            )
            .with_status(200)
            .with_body(
                r#"{"sha":"tree-sha","truncated":false,"tree":[{"path":"main.rs","mode":"100644","type":"blob","sha":"a"}]}"#,
            )
            .create();
        let _file = server
            .mock(
                "GET",
                format!("/repos/owner/repo/contents/src/main.rs?ref={TEST_SHA}").as_str(),
            )
            .match_header("accept", "application/vnd.github.raw")
            .with_status(200)
            .with_body("fn main() {}")
            .create();

        let dir = tempdir().expect("tempdir");
        let mut options = test_options(server.url(), "src", dir.path().join("downloads"));
        options.git_ref = None;
        let runner = test_runner(&server.url());

        let outcome = runner.run(&options).expect("download should succeed");
        assert_eq!(outcome.ref_sha.as_deref(), Some(TEST_SHA));
        assert_eq!(
            fs::read_to_string(outcome.saved_path.join("main.rs")).expect("file"),
            "fn main() {}"
        );
    }

    #[test]
    fn unresolvable_ref_fails_before_download() {
        let mut server = Server::new();
        let _root_tree = mock_root_tree(&mut server);
        let _commit = server
            .mock("GET", "/repos/owner/repo/commits/missing-branch")
            .with_status(404)
            .with_body(r#"{"message":"Not Found"}"#)
            .create();

        let mut options = test_options(
            server.url(),
            "README.md",
            PathBuf::from("/tmp/gh-download-missing-ref/README.md"),
        );
        options.git_ref = Some("missing-branch".to_string());
        let runner = test_runner(&server.url());

        let error = runner
            .run(&options)
            .expect_err("ref resolution should fail before downloading");
        assert!(matches!(error, AppError::HttpStatus { status: 404, .. }));
    }

    #[test]
    fn does_not_proxy_github_metadata_api_errors() {
        let mut api_server = Server::new();

        let _direct = api_server
            .mock("GET", "/repos/owner/repo/contents/README.md")
            .match_query(Matcher::Any)
            .match_header("accept", "application/vnd.github+json")
            .with_status(403)
            .with_body(r#"{"message":"rate limited"}"#)
            .create();

        let dir = tempdir().expect("tempdir");
        let options = ResolvedOptions {
            repo: "owner/repo".to_string(),
            remote_path: "README.md".to_string(),
            local_target: dir.path().join("README.md"),
            local_target_is_dir_hint: false,
            git_ref: Some(TEST_SHA.to_string()),
            token: None,
            api_base: api_server.url(),
            proxy_base: DEFAULT_GH_PROXY.to_string(),
            prefix_mode: PrefixProxyMode::Fallback,
            symlinks: SymlinkMode::Auto,
            lfs: LfsMode::Error,
            concurrency: 4,
            language: Language::En,
            overwrite: false,
            keep_going: false,
            json: false,
            debug: false,
            no_color: true,
        };
        let runner = Runner::new(
            RuntimeConfig {
                api_base: api_server.url(),
            },
            Output::new(false, Language::En),
        );

        let error = runner
            .run(&options)
            .expect_err("metadata API errors should not be retried through gh-proxy");
        assert!(matches!(error, AppError::HttpStatus { status: 403, .. }));
    }

    #[test]
    fn falls_back_to_proxy_for_retryable_anonymous_raw_download_errors() {
        let mut api_server = Server::new();
        let mut proxy_server = Server::new();
        let mut raw_server = Server::new();

        let download_url = format!("{}/owner/repo/main/README.md", raw_server.url());

        let _metadata = api_server
            .mock("GET", "/repos/owner/repo/contents/README.md")
            .match_query(Matcher::Any)
            .match_header("accept", "application/vnd.github+json")
            .with_status(200)
            .with_body(format!(
                r#"{{"name":"README.md","path":"README.md","type":"file","download_url":"{}"}}"#,
                download_url
            ))
            .create();

        let _direct_raw = raw_server
            .mock("GET", "/owner/repo/main/README.md")
            .with_status(503)
            .with_body("raw unavailable")
            .create();

        let proxied_path = format!("/{}", download_url);
        let _proxy = proxy_server
            .mock("GET", Matcher::Exact(proxied_path))
            .with_status(200)
            .with_body("proxy raw body")
            .create();

        let dir = tempdir().expect("tempdir");
        let options = ResolvedOptions {
            repo: "owner/repo".to_string(),
            remote_path: "README.md".to_string(),
            local_target: dir.path().join("README.md"),
            local_target_is_dir_hint: false,
            git_ref: Some(TEST_SHA.to_string()),
            token: None,
            api_base: api_server.url(),
            proxy_base: proxy_server.url(),
            prefix_mode: PrefixProxyMode::Fallback,
            symlinks: SymlinkMode::Auto,
            lfs: LfsMode::Error,
            concurrency: 4,
            language: Language::En,
            overwrite: false,
            keep_going: false,
            json: false,
            debug: false,
            no_color: true,
        };
        let runner = Runner::new(
            RuntimeConfig {
                api_base: api_server.url(),
            },
            Output::new(false, Language::En),
        );

        let outcome = runner.run(&options).expect("download should succeed");
        assert_eq!(
            fs::read_to_string(outcome.saved_path).expect("readme"),
            "proxy raw body"
        );
    }

    #[test]
    fn private_repo_with_token_uses_api_raw_without_anonymous_raw_attempt() {
        let mut server = Server::new();
        let _root_tree = mock_root_tree(&mut server);
        let download_url = format!("{}/download/README.md", server.url());

        let _repo = server
            .mock("GET", "/repos/owner/repo")
            .with_status(200)
            .with_body(r#"{"default_branch":"main","private":true}"#)
            .create();

        let _metadata = server
            .mock("GET", "/repos/owner/repo/contents/README.md")
            .match_query(Matcher::Any)
            .match_header("accept", "application/vnd.github+json")
            .with_status(200)
            .with_body(format!(
                r#"{{"name":"README.md","path":"README.md","type":"file","download_url":"{}"}}"#,
                download_url
            ))
            .create();

        let anonymous_raw = server
            .mock("GET", "/download/README.md")
            .expect(0)
            .with_status(200)
            .with_body("should not be requested")
            .create();

        let _raw_api = server
            .mock("GET", "/repos/owner/repo/contents/README.md")
            .match_query(Matcher::Any)
            .match_header("accept", "application/vnd.github.raw")
            .with_status(200)
            .with_body("private body")
            .create();

        let dir = tempdir().expect("tempdir");
        let target = dir.path().join("README.md");
        let options = ResolvedOptions {
            repo: "owner/repo".to_string(),
            remote_path: "README.md".to_string(),
            local_target: target.clone(),
            local_target_is_dir_hint: false,
            git_ref: Some(TEST_SHA.to_string()),
            token: Some("secret".to_string()),
            api_base: server.url(),
            proxy_base: DEFAULT_GH_PROXY.to_string(),
            prefix_mode: PrefixProxyMode::Fallback,
            symlinks: SymlinkMode::Auto,
            lfs: LfsMode::Error,
            concurrency: 4,
            language: Language::En,
            overwrite: false,
            keep_going: false,
            json: false,
            debug: false,
            no_color: true,
        };
        let runner = Runner::new(
            RuntimeConfig {
                api_base: server.url(),
            },
            Output::new(false, Language::En),
        );

        let outcome = runner.run(&options).expect("download should succeed");
        anonymous_raw.assert();
        assert_eq!(
            fs::read_to_string(outcome.saved_path).expect("file"),
            "private body"
        );
    }

    #[test]
    fn public_repo_with_token_can_retry_anonymous_raw_through_proxy() {
        let mut api_server = Server::new();
        let mut proxy_server = Server::new();
        let mut raw_server = Server::new();
        let download_url = format!("{}/owner/repo/main/README.md", raw_server.url());

        let _repo = api_server
            .mock("GET", "/repos/owner/repo")
            .with_status(200)
            .with_body(r#"{"default_branch":"main","private":false}"#)
            .create();

        let _metadata = api_server
            .mock("GET", "/repos/owner/repo/contents/README.md")
            .match_query(Matcher::Any)
            .match_header("accept", "application/vnd.github+json")
            .with_status(200)
            .with_body(format!(
                r#"{{"name":"README.md","path":"README.md","type":"file","download_url":"{}"}}"#,
                download_url
            ))
            .create();

        let _direct_raw = raw_server
            .mock("GET", "/owner/repo/main/README.md")
            .with_status(503)
            .with_body("raw unavailable")
            .create();

        let proxied_path = format!("/{}", download_url);
        let _proxy = proxy_server
            .mock("GET", Matcher::Exact(proxied_path))
            .with_status(200)
            .with_body("proxy raw body")
            .create();

        let dir = tempdir().expect("tempdir");
        let target = dir.path().join("README.md");
        let options = ResolvedOptions {
            repo: "owner/repo".to_string(),
            remote_path: "README.md".to_string(),
            local_target: target.clone(),
            local_target_is_dir_hint: false,
            git_ref: Some(TEST_SHA.to_string()),
            token: Some("secret".to_string()),
            api_base: api_server.url(),
            proxy_base: proxy_server.url(),
            prefix_mode: PrefixProxyMode::Fallback,
            symlinks: SymlinkMode::Auto,
            lfs: LfsMode::Error,
            concurrency: 4,
            language: Language::En,
            overwrite: false,
            keep_going: false,
            json: false,
            debug: false,
            no_color: true,
        };
        let runner = Runner::new(
            RuntimeConfig {
                api_base: api_server.url(),
            },
            Output::new(false, Language::En),
        );

        let outcome = runner.run(&options).expect("download should succeed");
        assert_eq!(
            fs::read_to_string(outcome.saved_path).expect("file"),
            "proxy raw body"
        );
    }

    #[test]
    fn prefer_mode_uses_prefix_proxy_first() {
        let mut api_server = Server::new();
        let mut proxy_server = Server::new();
        let mut raw_server = Server::new();

        let download_url = format!("{}/owner/repo/main/README.md", raw_server.url());

        let _metadata = api_server
            .mock("GET", "/repos/owner/repo/contents/README.md")
            .match_query(Matcher::Any)
            .match_header("accept", "application/vnd.github+json")
            .with_status(200)
            .with_body(format!(
                r#"{{"name":"README.md","path":"README.md","type":"file","download_url":"{}"}}"#,
                download_url
            ))
            .create();

        let _direct_raw = raw_server
            .mock("GET", "/owner/repo/main/README.md")
            .with_status(500)
            .with_body("should not be needed")
            .create();

        let proxied_path = format!("/{}", download_url);
        let _proxy = proxy_server
            .mock("GET", Matcher::Exact(proxied_path))
            .with_status(200)
            .with_body("prefer raw body")
            .create();

        let dir = tempdir().expect("tempdir");
        let options = ResolvedOptions {
            repo: "owner/repo".to_string(),
            remote_path: "README.md".to_string(),
            local_target: dir.path().join("README.md"),
            local_target_is_dir_hint: false,
            git_ref: Some(TEST_SHA.to_string()),
            token: None,
            api_base: api_server.url(),
            proxy_base: proxy_server.url(),
            prefix_mode: PrefixProxyMode::Prefer,
            symlinks: SymlinkMode::Auto,
            lfs: LfsMode::Error,
            concurrency: 4,
            language: Language::En,
            overwrite: false,
            keep_going: false,
            json: false,
            debug: false,
            no_color: true,
        };
        let runner = Runner::new(
            RuntimeConfig {
                api_base: api_server.url(),
            },
            Output::new(false, Language::En),
        );

        let outcome = runner.run(&options).expect("download should succeed");
        assert_eq!(
            fs::read_to_string(outcome.saved_path).expect("readme"),
            "prefer raw body"
        );
    }

    #[test]
    fn unparseable_recursive_tree_falls_back_to_non_recursive() {
        let mut server = Server::new();
        let _root_tree = mock_root_tree(&mut server);
        let _bad = server
            .mock(
                "GET",
                format!("/repos/owner/repo/git/trees/subtree-sha?recursive=1").as_str(),
            )
            .with_status(200)
            .with_body("{not valid json")
            .create();
        let _flat = server
            .mock(
                "GET",
                format!("/repos/owner/repo/git/trees/subtree-sha").as_str(),
            )
            .with_status(200)
            .with_body(
                r#"{"sha":"subtree-sha","truncated":false,"tree":[{"path":"main.rs","mode":"100644","type":"blob","sha":"a"}]}"#,
            )
            .create();

        let client = build_client().expect("client");
        let api_base = server.url();
        let options = test_options(api_base.clone(), "src", PathBuf::from("/tmp/downloads"));
        let runner = test_runner(&api_base);
        let mut stats = DownloadStats::default();
        let jobs = runner
            .collect_directory_jobs(
                &client,
                &options,
                &mut stats,
                "src",
                Path::new("/tmp/downloads/src"),
                TEST_SHA,
                false,
            )
            .expect("jobs should build");

        assert_eq!(jobs.len(), 1);
        assert_eq!(jobs[0].shown_path, "main.rs");
        _bad.assert();
        _flat.assert();
    }

    #[test]
    fn tree_enumeration_ignores_entries_outside_remote_root() {
        let mut server = Server::new();
        let _root_tree = server
            .mock(
                "GET",
                format!("/repos/owner/repo/git/trees/{TEST_SHA}").as_str(),
            )
            .with_status(200)
            .with_body(
                r#"{"sha":"tree-sha","truncated":false,"tree":[{"path":"other","mode":"040000","type":"tree","sha":"other-sha"},{"path":"src","mode":"040000","type":"tree","sha":"subtree-sha"}]}"#,
            )
            .create();
        let _tree = server
            .mock(
                "GET",
                format!("/repos/owner/repo/git/trees/subtree-sha?recursive=1").as_str(),
            )
            .with_status(200)
            .with_body(
                r#"{"sha":"subtree-sha","truncated":false,"tree":[{"path":"main.rs","mode":"100644","type":"blob","sha":"a"}]}"#,
            )
            .create();

        let client = build_client().expect("client");
        let api_base = server.url();
        let options = test_options(api_base.clone(), "src", PathBuf::from("/tmp/downloads"));
        let runner = test_runner(&api_base);
        let mut stats = DownloadStats::default();
        let jobs = runner
            .collect_directory_jobs(
                &client,
                &options,
                &mut stats,
                "src",
                Path::new("/tmp/downloads/src"),
                TEST_SHA,
                false,
            )
            .expect("jobs should build");

        assert_eq!(jobs.len(), 1);
        assert_eq!(jobs[0].shown_path, "main.rs");
    }

    #[test]
    fn symlink_auto_follows_file_target() {
        let mut server = Server::new();
        let _root_tree = mock_root_tree(&mut server);
        let _listing = server
            .mock("GET", "/repos/owner/repo/contents/src")
            .match_query(Matcher::Any)
            .with_status(200)
            .with_body("[]")
            .create();
        let _tree = server
            .mock(
                "GET",
                format!("/repos/owner/repo/git/trees/subtree-sha?recursive=1").as_str(),
            )
            .with_status(200)
            .with_body(
                r#"{"sha":"tree-sha","truncated":false,"tree":[{"path":"link","mode":"120000","type":"blob","sha":"blob-sha"}]}"#,
            )
            .create();
        let _symlink_metadata = server
            .mock(
                "GET",
                format!("/repos/owner/repo/contents/src/link?ref={TEST_SHA}").as_str(),
            )
            .match_header("accept", "application/vnd.github+json")
            .with_status(200)
            .with_body(r#"{"type":"file","name":"link"}"#)
            .create();
        let _content = server
            .mock(
                "GET",
                format!("/repos/owner/repo/contents/src/link?ref={TEST_SHA}").as_str(),
            )
            .match_header("accept", "application/vnd.github.raw")
            .with_status(200)
            .with_body("target content")
            .create();

        let dir = tempdir().expect("tempdir");
        let options = test_options(server.url(), "src", dir.path().join("downloads"));
        let runner = test_runner(&server.url());

        let outcome = runner.run(&options).expect("download should succeed");
        assert_eq!(outcome.stats.files_downloaded, 1);
        assert_eq!(outcome.stats.skipped_symlinks, 0);
        assert_eq!(
            fs::read_to_string(outcome.saved_path.join("link")).expect("content"),
            "target content"
        );
    }

    #[cfg(unix)]
    #[test]
    fn symlink_link_mode_creates_local_symlink() {
        let mut server = Server::new();
        let _root_tree = mock_root_tree(&mut server);
        let _listing = server
            .mock("GET", "/repos/owner/repo/contents/src")
            .match_query(Matcher::Any)
            .with_status(200)
            .with_body("[]")
            .create();
        let _tree = server
            .mock(
                "GET",
                format!("/repos/owner/repo/git/trees/subtree-sha?recursive=1").as_str(),
            )
            .with_status(200)
            .with_body(
                r#"{"sha":"tree-sha","truncated":false,"tree":[{"path":"link","mode":"120000","type":"blob","sha":"blob-sha"}]}"#,
            )
            .create();
        let _blob = server
            .mock("GET", "/repos/owner/repo/git/blobs/blob-sha")
            .match_header("accept", "application/vnd.github.raw")
            .with_status(200)
            .with_body("../target.txt")
            .create();

        let dir = tempdir().expect("tempdir");
        let mut options = test_options(server.url(), "src", dir.path().join("downloads"));
        options.symlinks = SymlinkMode::Link;
        let runner = test_runner(&server.url());

        let outcome = runner.run(&options).expect("download should succeed");
        let link = outcome.saved_path.join("link");
        let metadata = fs::symlink_metadata(&link).expect("symlink metadata");
        assert!(metadata.file_type().is_symlink());
        assert_eq!(
            fs::read_link(&link).expect("link target"),
            PathBuf::from("../target.txt")
        );
    }

    #[test]
    fn lfs_pointer_fails_by_default() {
        let mut server = Server::new();
        let _root_tree = mock_root_tree(&mut server);
        let download_url = format!("{}/download/README.md", server.url());
        let _metadata = server
            .mock("GET", "/repos/owner/repo/contents/README.md")
            .match_query(Matcher::Any)
            .match_header("accept", "application/vnd.github+json")
            .with_status(200)
            .with_body(format!(
                r#"{{"name":"README.md","path":"README.md","type":"file","download_url":"{}"}}"#,
                download_url
            ))
            .create();
        let _pointer = server
            .mock("GET", "/download/README.md")
            .with_status(200)
            .with_body("version https://git-lfs.github.com/spec/v1\noid sha256:abc\nsize 26454\n")
            .create();

        let dir = tempdir().expect("tempdir");
        let target = dir.path().join("README.md");
        let options = test_options(server.url(), "README.md", target.clone());
        let runner = test_runner(&server.url());

        let error = runner
            .run(&options)
            .expect_err("LFS pointer should not be saved by default");
        assert!(matches!(error, AppError::LfsObject { .. }));
        assert!(!target.exists());
    }

    #[test]
    fn lfs_pointer_detection_recognizes_pointer_text() {
        let dir = tempdir().expect("tempdir");
        let pointer = dir.path().join("pointer.bin");
        fs::write(
            &pointer,
            "version https://git-lfs.github.com/spec/v1\noid sha256:abc\nsize 26454\n",
        )
        .expect("write pointer");
        assert!(detect_lfs_pointer(&pointer).expect("detect").is_some());

        let normal = dir.path().join("normal.bin");
        fs::write(&normal, "ordinary content").expect("write normal");
        assert!(detect_lfs_pointer(&normal).expect("detect").is_none());
    }

    fn mock_root_tree(server: &mut mockito::ServerGuard) -> mockito::Mock {
        server
            .mock(
                "GET",
                format!("/repos/owner/repo/git/trees/{TEST_SHA}").as_str(),
            )
            .with_status(200)
            .with_body(
                r#"{"sha":"tree-sha","truncated":false,"tree":[{"path":"src","mode":"040000","type":"tree","sha":"subtree-sha"}]}"#,
            )
            .create()
    }

    fn directory_failure_server() -> (mockito::ServerGuard, mockito::Mock) {
        let mut server = Server::new();
        let _root_tree = mock_root_tree(&mut server);
        let _listing = server
            .mock("GET", "/repos/owner/repo/contents/src")
            .match_query(Matcher::Any)
            .match_header("accept", "application/vnd.github+json")
            .with_status(200)
            .with_body("[]")
            .create();
        let _repo = server
            .mock("GET", "/repos/owner/repo")
            .with_status(200)
            .with_body(r#"{"default_branch":"main","private":false}"#)
            .create();
        let _tree = server
            .mock("GET", format!("/repos/owner/repo/git/trees/subtree-sha?recursive=1").as_str())
            .with_status(200)
            .with_body(
                r#"{"sha":"tree-sha","truncated":false,"tree":[{"path":"a.rs","mode":"100644","type":"blob","sha":"a"},{"path":"b.rs","mode":"100644","type":"blob","sha":"b"},{"path":"c.rs","mode":"100644","type":"blob","sha":"c"}]}"#,
            )
            .create();
        let _a = server
            .mock(
                "GET",
                format!("/repos/owner/repo/contents/src/a.rs?ref={TEST_SHA}").as_str(),
            )
            .match_header("accept", "application/vnd.github.raw")
            .with_status(200)
            .with_body("fn a() {}")
            .create();
        let _b = server
            .mock(
                "GET",
                format!("/repos/owner/repo/contents/src/b.rs?ref={TEST_SHA}").as_str(),
            )
            .match_header("accept", "application/vnd.github.raw")
            .with_status(503)
            .with_body("boom")
            .create();
        let c = server
            .mock(
                "GET",
                format!("/repos/owner/repo/contents/src/c.rs?ref={TEST_SHA}").as_str(),
            )
            .match_header("accept", "application/vnd.github.raw")
            .with_status(200)
            .with_body("fn c() {}")
            .create();
        (server, c)
    }

    fn failing_directory_options(
        api_base: String,
        target: PathBuf,
        keep_going: bool,
    ) -> ResolvedOptions {
        ResolvedOptions {
            repo: "owner/repo".to_string(),
            remote_path: "src".to_string(),
            local_target: target,
            local_target_is_dir_hint: false,
            git_ref: Some(TEST_SHA.to_string()),
            token: None,
            api_base,
            proxy_base: DEFAULT_GH_PROXY.to_string(),
            prefix_mode: PrefixProxyMode::Fallback,
            symlinks: SymlinkMode::Auto,
            lfs: LfsMode::Error,
            concurrency: 1,
            language: Language::En,
            overwrite: false,
            keep_going,
            json: false,
            debug: false,
            no_color: true,
        }
    }

    #[test]
    fn keep_going_continues_after_directory_file_failure() {
        let (server, c_mock) = directory_failure_server();
        let dir = tempdir().expect("tempdir");
        let options = failing_directory_options(server.url(), dir.path().join("downloads"), true);
        let runner = Runner::new(
            RuntimeConfig {
                api_base: server.url(),
            },
            Output::new(false, Language::En),
        );

        let error = runner
            .run(&options)
            .expect_err("partial directory failure should fail the command");

        match error {
            AppError::DirectoryTransfer {
                failures,
                stats,
                aborted_early,
                ..
            } => {
                assert_eq!(failures.len(), 1);
                assert_eq!(failures[0].path, "src/b.rs");
                assert!(failures[0].reason.contains("503"));
                assert_eq!(stats.downloaded, 2);
                assert!(!aborted_early);
            }
            other => panic!("expected aggregate directory failure, got {other:?}"),
        }
        c_mock.assert();
    }

    #[test]
    fn fail_fast_aborts_remaining_directory_files() {
        let mut server = Server::new();
        let _root_tree = mock_root_tree(&mut server);
        let _listing = server
            .mock("GET", "/repos/owner/repo/contents/src")
            .match_query(Matcher::Any)
            .match_header("accept", "application/vnd.github+json")
            .with_status(200)
            .with_body("[]")
            .create();
        let _repo = server
            .mock("GET", "/repos/owner/repo")
            .with_status(200)
            .with_body(r#"{"default_branch":"main","private":false}"#)
            .create();
        let _tree = server
            .mock("GET", format!("/repos/owner/repo/git/trees/subtree-sha?recursive=1").as_str())
            .with_status(200)
            .with_body(
                r#"{"sha":"tree-sha","truncated":false,"tree":[{"path":"a.rs","mode":"100644","type":"blob","sha":"a"},{"path":"b.rs","mode":"100644","type":"blob","sha":"b"},{"path":"c.rs","mode":"100644","type":"blob","sha":"c"}]}"#,
            )
            .create();
        let _a = server
            .mock(
                "GET",
                format!("/repos/owner/repo/contents/src/a.rs?ref={TEST_SHA}").as_str(),
            )
            .match_header("accept", "application/vnd.github.raw")
            .with_status(200)
            .with_body("fn a() {}")
            .create();
        let _b = server
            .mock(
                "GET",
                format!("/repos/owner/repo/contents/src/b.rs?ref={TEST_SHA}").as_str(),
            )
            .match_header("accept", "application/vnd.github.raw")
            .with_status(503)
            .with_body("boom")
            .create();
        let _c = server
            .mock(
                "GET",
                format!("/repos/owner/repo/contents/src/c.rs?ref={TEST_SHA}").as_str(),
            )
            .match_header("accept", "application/vnd.github.raw")
            .expect(0)
            .with_status(200)
            .with_body("fn c() {}")
            .create();

        let dir = tempdir().expect("tempdir");
        let options = failing_directory_options(server.url(), dir.path().join("downloads"), false);
        let runner = Runner::new(
            RuntimeConfig {
                api_base: server.url(),
            },
            Output::new(false, Language::En),
        );

        let error = runner
            .run(&options)
            .expect_err("fail-fast directory failure should fail the command");

        match error {
            AppError::DirectoryTransfer {
                failures,
                stats,
                aborted_early,
                ..
            } => {
                assert_eq!(failures.len(), 1);
                assert_eq!(stats.downloaded, 1);
                assert!(aborted_early);
            }
            other => panic!("expected aggregate directory failure, got {other:?}"),
        }
    }

    #[test]
    fn bounded_work_uses_requested_concurrency() {
        let in_flight = Arc::new(AtomicUsize::new(0));
        let max_in_flight = Arc::new(AtomicUsize::new(0));
        let jobs: Vec<_> = (0..6).collect();

        let outcome = run_bounded_work(&jobs, 2, false, &|_| {
            let current = in_flight.fetch_add(1, Ordering::SeqCst) + 1;
            max_in_flight.fetch_max(current, Ordering::SeqCst);
            thread::sleep(Duration::from_millis(20));
            in_flight.fetch_sub(1, Ordering::SeqCst);
            Ok::<(), AppError>(())
        });

        assert_eq!(outcome.results.len(), 6);
        assert!(outcome.errors.is_empty());
        assert!(!outcome.aborted_early);
        assert_eq!(max_in_flight.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn bounded_work_fail_fast_stops_after_failing_batch() {
        let jobs: Vec<_> = (0..6).collect();

        let outcome = run_bounded_work(&jobs, 2, false, &|job| {
            if job == 1 {
                Err(AppError::InvalidPath("boom".to_string()))
            } else {
                Ok(())
            }
        });

        assert_eq!(outcome.results.len(), 1);
        assert_eq!(outcome.errors.len(), 1);
        assert!(outcome.aborted_early);
    }

    #[test]
    fn bounded_work_keep_going_collects_all_failures() {
        let jobs: Vec<_> = (0..6).collect();

        let outcome = run_bounded_work(&jobs, 2, true, &|job| {
            if job % 2 == 1 {
                Err(AppError::InvalidPath(format!("boom-{job}")))
            } else {
                Ok(())
            }
        });

        assert_eq!(outcome.results.len(), 3);
        assert_eq!(outcome.errors.len(), 3);
        assert!(!outcome.aborted_early);
    }

    #[test]
    fn effective_directory_worker_threads_uses_file_count_and_configured_bound() {
        assert_eq!(effective_directory_worker_threads(11, 4), 4);
        assert_eq!(effective_directory_worker_threads(3, 8), 3);
        assert_eq!(effective_directory_worker_threads(0, 4), 0);
    }

    #[test]
    fn tree_enumeration_preserves_relative_paths_and_rev() {
        let mut server = Server::new();
        let _root_tree = mock_root_tree(&mut server);
        let _tree = server
            .mock("GET", format!("/repos/owner/repo/git/trees/subtree-sha?recursive=1").as_str())
            .with_status(200)
            .with_body(
                r#"{"sha":"tree-sha","truncated":false,"tree":[{"path":"main.rs","mode":"100644","type":"blob","sha":"a","size":12},{"path":"nested/lib.rs","mode":"100755","type":"blob","sha":"b","size":18}]}"#,
            )
            .create();

        let client = build_client().expect("client");
        let api_base = server.url();
        let options = test_options(api_base.clone(), "src", PathBuf::from("/tmp/downloads"));
        let runner = test_runner(&api_base);
        let mut stats = DownloadStats::default();
        let jobs = runner
            .collect_directory_jobs(
                &client,
                &options,
                &mut stats,
                "src",
                Path::new("/tmp/downloads/src"),
                TEST_SHA,
                false,
            )
            .expect("jobs should build");

        assert_eq!(jobs.len(), 2);
        assert_eq!(jobs[0].shown_path, "main.rs");
        assert_eq!(
            jobs[0].local_target,
            PathBuf::from("/tmp/downloads/src/main.rs")
        );
        assert_eq!(jobs[0].rev.as_deref(), Some(TEST_SHA));
        assert_eq!(jobs[0].item.download_url, None);
        assert_eq!(jobs[1].shown_path, "nested/lib.rs");
        assert_eq!(
            jobs[1].local_target,
            PathBuf::from("/tmp/downloads/src/nested/lib.rs")
        );
    }

    #[test]
    fn tree_enumeration_reads_more_than_contents_api_limit() {
        let mut server = Server::new();
        let _root_tree = mock_root_tree(&mut server);
        let entries: Vec<String> = (0..1001)
            .map(|index| {
                format!(
                    r#"{{"path":"f{}.txt","mode":"100644","type":"blob","sha":"s{}","size":3}}"#,
                    index, index
                )
            })
            .collect();
        let body = format!(
            r#"{{"sha":"tree-sha","truncated":false,"tree":[{}]}}"#,
            entries.join(",")
        );

        let _tree = server
            .mock(
                "GET",
                format!("/repos/owner/repo/git/trees/subtree-sha?recursive=1").as_str(),
            )
            .with_status(200)
            .with_body(body)
            .create();

        let client = build_client().expect("client");
        let api_base = server.url();
        let options = test_options(api_base.clone(), "src", PathBuf::from("/tmp/downloads"));
        let runner = test_runner(&api_base);
        let mut stats = DownloadStats::default();
        let jobs = runner
            .collect_directory_jobs(
                &client,
                &options,
                &mut stats,
                "src",
                Path::new("/tmp/downloads/src"),
                TEST_SHA,
                false,
            )
            .expect("jobs should build");

        assert_eq!(jobs.len(), 1001);
    }

    #[test]
    fn truncated_tree_falls_back_to_per_subtree_requests() {
        let mut server = Server::new();
        let _root_tree = mock_root_tree(&mut server);
        let _truncated = server
            .mock("GET", format!("/repos/owner/repo/git/trees/subtree-sha?recursive=1").as_str())
            .with_status(200)
            .with_body(
                r#"{"sha":"subtree-sha","truncated":true,"tree":[{"path":"nested","mode":"040000","type":"tree","sha":"nested-sha"}]}"#,
            )
            .create();
        let _root = server
            .mock("GET", format!("/repos/owner/repo/git/trees/subtree-sha").as_str())
            .with_status(200)
            .with_body(
                r#"{"sha":"subtree-sha","truncated":false,"tree":[{"path":"nested","mode":"040000","type":"tree","sha":"nested-sha"}]}"#,
            )
            .create();
        let _nested = server
            .mock("GET", "/repos/owner/repo/git/trees/nested-sha?recursive=1")
            .with_status(200)
            .with_body(
                r#"{"sha":"nested-sha","truncated":false,"tree":[{"path":"lib.rs","mode":"100644","type":"blob","sha":"a","size":12}]}"#,
            )
            .create();

        let client = build_client().expect("client");
        let api_base = server.url();
        let options = test_options(api_base.clone(), "src", PathBuf::from("/tmp/downloads"));
        let runner = test_runner(&api_base);
        let mut stats = DownloadStats::default();
        let jobs = runner
            .collect_directory_jobs(
                &client,
                &options,
                &mut stats,
                "src",
                Path::new("/tmp/downloads/src"),
                TEST_SHA,
                false,
            )
            .expect("jobs should build");

        assert_eq!(jobs.len(), 1);
        assert_eq!(jobs[0].shown_path, "nested/lib.rs");
    }

    #[test]
    fn tree_enumeration_skips_symlinks_and_submodules() {
        let mut server = Server::new();
        let _root_tree = mock_root_tree(&mut server);
        let _tree = server
            .mock("GET", format!("/repos/owner/repo/git/trees/subtree-sha?recursive=1").as_str())
            .with_status(200)
            .with_body(
                r#"{"sha":"tree-sha","truncated":false,"tree":[{"path":"main.rs","mode":"100644","type":"blob","sha":"a"},{"path":"link","mode":"120000","type":"blob","sha":"b"},{"path":"sub","mode":"160000","type":"commit","sha":"c"}]}"#,
            )
            .create();

        let _submodule = server
            .mock(
                "GET",
                format!("/repos/owner/repo/contents/src/sub?ref={TEST_SHA}").as_str(),
            )
            .with_status(200)
            .with_body(
                r#"{"type":"submodule","submodule_git_url":"https://example.invalid/sub.git"}"#,
            )
            .create();

        let client = build_client().expect("client");
        let api_base = server.url();
        let mut options = test_options(api_base.clone(), "src", PathBuf::from("/tmp/downloads"));
        options.symlinks = SymlinkMode::Skip;
        let runner = test_runner(&api_base);
        let mut stats = DownloadStats::default();
        let jobs = runner
            .collect_directory_jobs(
                &client,
                &options,
                &mut stats,
                "src",
                Path::new("/tmp/downloads/src"),
                TEST_SHA,
                false,
            )
            .expect("jobs should build");

        assert_eq!(jobs.len(), 1);
        assert_eq!(jobs[0].shown_path, "main.rs");
        assert_eq!(stats.skipped_symlinks, 1);
        assert_eq!(stats.skipped_submodules, 1);
        assert_eq!(stats.skipped_entries, 0);
    }

    #[test]
    fn skips_existing_direct_file_by_default() {
        let mut server = Server::new();
        let _root_tree = mock_root_tree(&mut server);
        let download_url = format!("{}/downloads/README.md", server.url());

        let _metadata = server
            .mock("GET", "/repos/owner/repo/contents/README.md")
            .match_query(Matcher::Any)
            .match_header("accept", "application/vnd.github+json")
            .with_status(200)
            .with_body(format!(
                r#"{{"name":"README.md","path":"README.md","type":"file","download_url":"{}"}}"#,
                download_url
            ))
            .create();

        let download = server
            .mock("GET", "/downloads/README.md")
            .expect(0)
            .with_status(200)
            .with_body("new body")
            .create();

        let dir = tempdir().expect("tempdir");
        let target = dir.path().join("README.md");
        fs::write(&target, "keep existing").expect("seed existing file");

        let options = ResolvedOptions {
            repo: "owner/repo".to_string(),
            remote_path: "README.md".to_string(),
            local_target: target.clone(),
            local_target_is_dir_hint: false,
            git_ref: Some(TEST_SHA.to_string()),
            token: None,
            api_base: server.url(),
            proxy_base: DEFAULT_GH_PROXY.to_string(),
            prefix_mode: PrefixProxyMode::Fallback,
            symlinks: SymlinkMode::Auto,
            lfs: LfsMode::Error,
            concurrency: 4,
            language: Language::En,
            overwrite: false,
            keep_going: false,
            json: false,
            debug: false,
            no_color: true,
        };
        let runner = Runner::new(
            RuntimeConfig {
                api_base: server.url(),
            },
            Output::new(false, Language::En),
        );

        let outcome = runner.run(&options).expect("download should succeed");
        download.assert();
        assert_eq!(outcome.saved_path, target);
        assert_eq!(outcome.stats.files_downloaded, 0);
        assert_eq!(outcome.stats.skipped_existing_files, 1);
        assert_eq!(
            fs::read_to_string(outcome.saved_path).expect("existing file"),
            "keep existing"
        );
    }

    #[test]
    fn overwrites_existing_direct_file_when_enabled() {
        let mut server = Server::new();
        let _root_tree = mock_root_tree(&mut server);
        let download_url = format!("{}/downloads/README.md", server.url());

        let _metadata = server
            .mock("GET", "/repos/owner/repo/contents/README.md")
            .match_query(Matcher::Any)
            .match_header("accept", "application/vnd.github+json")
            .with_status(200)
            .with_body(format!(
                r#"{{"name":"README.md","path":"README.md","type":"file","download_url":"{}"}}"#,
                download_url
            ))
            .create();

        let download = server
            .mock("GET", "/downloads/README.md")
            .expect(1)
            .with_status(200)
            .with_body("new body")
            .create();

        let dir = tempdir().expect("tempdir");
        let target = dir.path().join("README.md");
        fs::write(&target, "keep existing").expect("seed existing file");

        let options = ResolvedOptions {
            repo: "owner/repo".to_string(),
            remote_path: "README.md".to_string(),
            local_target: target.clone(),
            local_target_is_dir_hint: false,
            git_ref: Some(TEST_SHA.to_string()),
            token: None,
            api_base: server.url(),
            proxy_base: DEFAULT_GH_PROXY.to_string(),
            prefix_mode: PrefixProxyMode::Fallback,
            symlinks: SymlinkMode::Auto,
            lfs: LfsMode::Error,
            concurrency: 4,
            language: Language::En,
            overwrite: true,
            keep_going: false,
            json: false,
            debug: false,
            no_color: true,
        };
        let runner = Runner::new(
            RuntimeConfig {
                api_base: server.url(),
            },
            Output::new(false, Language::En),
        );

        let outcome = runner.run(&options).expect("download should succeed");
        download.assert();
        assert_eq!(outcome.stats.files_downloaded, 1);
        assert_eq!(outcome.stats.skipped_existing_files, 0);
        assert_eq!(
            fs::read_to_string(outcome.saved_path).expect("overwritten file"),
            "new body"
        );
    }

    #[test]
    fn skips_existing_directory_files_by_default() {
        let mut server = Server::new();
        let _root_tree = mock_root_tree(&mut server);

        let _listing = server
            .mock("GET", "/repos/owner/repo/contents/src")
            .match_query(Matcher::Any)
            .match_header("accept", "application/vnd.github+json")
            .with_status(200)
            .with_body("[]")
            .create();

        let _repo = server
            .mock("GET", "/repos/owner/repo")
            .with_status(200)
            .with_body(r#"{"default_branch":"main","private":false}"#)
            .create();

        let _tree = server
            .mock("GET", format!("/repos/owner/repo/git/trees/subtree-sha?recursive=1").as_str())
            .with_status(200)
            .with_body(
                r#"{"sha":"tree-sha","truncated":false,"tree":[{"path":"existing.rs","mode":"100644","type":"blob","sha":"a","size":12},{"path":"new.rs","mode":"100644","type":"blob","sha":"b","size":13}]}"#,
            )
            .create();

        let existing_download = server
            .mock(
                "GET",
                format!("/repos/owner/repo/contents/src/existing.rs?ref={TEST_SHA}").as_str(),
            )
            .match_header("accept", "application/vnd.github.raw")
            .expect(0)
            .with_status(200)
            .with_body("should not be downloaded")
            .create();

        let new_download = server
            .mock(
                "GET",
                format!("/repos/owner/repo/contents/src/new.rs?ref={TEST_SHA}").as_str(),
            )
            .match_header("accept", "application/vnd.github.raw")
            .expect(1)
            .with_status(200)
            .with_body("pub fn fresh() {}")
            .create();

        let dir = tempdir().expect("tempdir");
        let directory_target = dir.path().join("downloads/src");
        fs::create_dir_all(&directory_target).expect("create target dir");
        fs::write(directory_target.join("existing.rs"), "keep existing")
            .expect("seed existing file");

        let options = ResolvedOptions {
            repo: "owner/repo".to_string(),
            remote_path: "src".to_string(),
            local_target: dir.path().join("downloads"),
            local_target_is_dir_hint: false,
            git_ref: Some(TEST_SHA.to_string()),
            token: None,
            api_base: server.url(),
            proxy_base: DEFAULT_GH_PROXY.to_string(),
            prefix_mode: PrefixProxyMode::Fallback,
            symlinks: SymlinkMode::Auto,
            lfs: LfsMode::Error,
            concurrency: 4,
            language: Language::En,
            overwrite: false,
            keep_going: false,
            json: false,
            debug: false,
            no_color: true,
        };
        let runner = Runner::new(
            RuntimeConfig {
                api_base: server.url(),
            },
            Output::new(false, Language::En),
        );

        let outcome = runner.run(&options).expect("download should succeed");
        existing_download.assert();
        new_download.assert();
        assert_eq!(outcome.stats.files_downloaded, 1);
        assert_eq!(outcome.stats.skipped_existing_files, 1);
        assert_eq!(
            fs::read_to_string(outcome.saved_path.join("existing.rs")).expect("existing"),
            "keep existing"
        );
        assert_eq!(
            fs::read_to_string(outcome.saved_path.join("new.rs")).expect("new"),
            "pub fn fresh() {}"
        );
    }

    #[test]
    fn overwrites_existing_directory_files_when_enabled() {
        let mut server = Server::new();
        let _root_tree = mock_root_tree(&mut server);

        let _listing = server
            .mock("GET", "/repos/owner/repo/contents/src")
            .match_query(Matcher::Any)
            .match_header("accept", "application/vnd.github+json")
            .with_status(200)
            .with_body("[]")
            .create();

        let _repo = server
            .mock("GET", "/repos/owner/repo")
            .with_status(200)
            .with_body(r#"{"default_branch":"main","private":false}"#)
            .create();

        let _tree = server
            .mock("GET", format!("/repos/owner/repo/git/trees/subtree-sha?recursive=1").as_str())
            .with_status(200)
            .with_body(
                r#"{"sha":"tree-sha","truncated":false,"tree":[{"path":"existing.rs","mode":"100644","type":"blob","sha":"a","size":12},{"path":"new.rs","mode":"100644","type":"blob","sha":"b","size":13}]}"#,
            )
            .create();

        let existing_download = server
            .mock(
                "GET",
                format!("/repos/owner/repo/contents/src/existing.rs?ref={TEST_SHA}").as_str(),
            )
            .match_header("accept", "application/vnd.github.raw")
            .expect(1)
            .with_status(200)
            .with_body("pub fn replaced() {}")
            .create();

        let new_download = server
            .mock(
                "GET",
                format!("/repos/owner/repo/contents/src/new.rs?ref={TEST_SHA}").as_str(),
            )
            .match_header("accept", "application/vnd.github.raw")
            .expect(1)
            .with_status(200)
            .with_body("pub fn fresh() {}")
            .create();

        let dir = tempdir().expect("tempdir");
        let directory_target = dir.path().join("downloads/src");
        fs::create_dir_all(&directory_target).expect("create target dir");
        fs::write(directory_target.join("existing.rs"), "keep existing")
            .expect("seed existing file");

        let options = ResolvedOptions {
            repo: "owner/repo".to_string(),
            remote_path: "src".to_string(),
            local_target: dir.path().join("downloads"),
            local_target_is_dir_hint: false,
            git_ref: Some(TEST_SHA.to_string()),
            token: None,
            api_base: server.url(),
            proxy_base: DEFAULT_GH_PROXY.to_string(),
            prefix_mode: PrefixProxyMode::Fallback,
            symlinks: SymlinkMode::Auto,
            lfs: LfsMode::Error,
            concurrency: 4,
            language: Language::En,
            overwrite: true,
            keep_going: false,
            json: false,
            debug: false,
            no_color: true,
        };
        let runner = Runner::new(
            RuntimeConfig {
                api_base: server.url(),
            },
            Output::new(false, Language::En),
        );

        let outcome = runner.run(&options).expect("download should succeed");
        existing_download.assert();
        new_download.assert();
        assert_eq!(outcome.stats.files_downloaded, 2);
        assert_eq!(outcome.stats.skipped_existing_files, 0);
        assert_eq!(
            fs::read_to_string(outcome.saved_path.join("existing.rs")).expect("existing"),
            "pub fn replaced() {}"
        );
    }

    #[test]
    fn uses_custom_api_base_for_metadata_requests() {
        let mut api_server = Server::new();
        let mut raw_server = Server::new();

        let download_url = format!("{}/raw/README.md", raw_server.url());

        let _metadata = api_server
            .mock(
                "GET",
                "/enterprise/api/v3/repos/owner/repo/contents/README.md",
            )
            .match_query(Matcher::Any)
            .match_header("accept", "application/vnd.github+json")
            .with_status(200)
            .with_body(format!(
                r#"{{"name":"README.md","path":"README.md","type":"file","download_url":"{}"}}"#,
                download_url
            ))
            .create();

        let _download = raw_server
            .mock("GET", "/raw/README.md")
            .with_status(200)
            .with_body("enterprise body")
            .create();

        let dir = tempdir().expect("tempdir");
        let target = dir.path().join("README.md");
        let options = ResolvedOptions {
            repo: "owner/repo".to_string(),
            remote_path: "README.md".to_string(),
            local_target: target.clone(),
            local_target_is_dir_hint: false,
            git_ref: Some(TEST_SHA.to_string()),
            token: None,
            api_base: format!("{}/enterprise/api/v3/", api_server.url()),
            proxy_base: DEFAULT_GH_PROXY.to_string(),
            prefix_mode: PrefixProxyMode::Direct,
            symlinks: SymlinkMode::Auto,
            lfs: LfsMode::Error,
            concurrency: 4,
            language: Language::En,
            overwrite: false,
            keep_going: false,
            json: false,
            debug: false,
            no_color: true,
        };
        let runner = Runner::new(
            RuntimeConfig {
                api_base: options.api_base.clone(),
            },
            Output::new(false, Language::En),
        );

        let outcome = runner.run(&options).expect("download should succeed");
        assert_eq!(outcome.saved_path, target);
        assert_eq!(fs::read_to_string(target).expect("file"), "enterprise body");
    }
}
