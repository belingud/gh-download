mod models;
mod paths;
mod raw;
mod transport;

use std::fs;
use std::path::{Path, PathBuf};
use std::thread;

use reqwest::blocking::Client;
use serde_json::Value;

use crate::cli::{PrefixProxyMode, ResolvedOptions};
use crate::error::AppError;
use crate::i18n::Language;
use crate::output::Output;

use self::models::ContentItem;
pub use self::paths::{
    build_contents_api_url, choose_directory_target, format_remote_path, join_proxy_url,
    normalize_repo_path, relative_item_path,
};
use self::paths::{choose_file_target, file_name_from_remote_path, redact_url_for_display};
use self::raw::{RawDownloadStrategy, should_attempt_prefix_proxy};
use self::transport::{build_client, send_json_request, stream_download};

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
}

#[derive(Debug)]
pub struct RunOutcome {
    pub saved_path: PathBuf,
    pub stats: DownloadStats,
}

#[derive(Debug, Clone)]
struct DownloadJob {
    item: ContentItem,
    local_target: PathBuf,
    shown_path: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SaveFileOutcome {
    Downloaded,
    SkippedExisting,
}

pub struct Runner {
    config: RuntimeConfig,
    output: Output,
}

impl Runner {
    pub fn new(config: RuntimeConfig, output: Output) -> Self {
        Self { config, output }
    }

    pub fn run(&self, options: &ResolvedOptions) -> Result<RunOutcome, AppError> {
        let client = build_client()?;
        let mut stats = DownloadStats::default();
        let saved_path = self.detect_and_download(&client, options, &mut stats)?;
        self.output.completion(
            &options.repo,
            &options.remote_path,
            &saved_path,
            stats.files_downloaded,
            stats.skipped_existing_files,
            stats.skipped_entries,
        );
        Ok(RunOutcome { saved_path, stats })
    }

    fn detect_and_download(
        &self,
        client: &Client,
        options: &ResolvedOptions,
        stats: &mut DownloadStats,
    ) -> Result<PathBuf, AppError> {
        let normalized_path = normalize_repo_path(&options.remote_path);
        let metadata_url = build_contents_api_url(
            &self.config.api_base,
            &options.repo,
            &normalized_path,
            options.git_ref.as_deref(),
        );
        self.debug_log_url(options, "metadata-url", &metadata_url);
        let response = self.get_json(client, &metadata_url, options)?;

        if response.is_object() {
            let item: ContentItem =
                serde_json::from_value(response).map_err(|err| AppError::Json(err.to_string()))?;
            let final_target = choose_file_target(&options.local_target, &normalized_path, &item);
            let shown_path = item
                .name
                .clone()
                .unwrap_or_else(|| file_name_from_remote_path(&normalized_path));
            match self.save_file(client, options, &item, &final_target, &shown_path)? {
                SaveFileOutcome::Downloaded => stats.files_downloaded += 1,
                SaveFileOutcome::SkippedExisting => stats.skipped_existing_files += 1,
            }
            return Ok(final_target);
        }

        if response.is_array() {
            let directory_target = choose_directory_target(&options.local_target, &normalized_path);
            let entries: Vec<ContentItem> =
                serde_json::from_value(response).map_err(|err| AppError::Json(err.to_string()))?;
            let files = self.collect_directory_files(
                client,
                options,
                stats,
                &normalized_path,
                Some(entries),
            )?;
            let worker_threads =
                effective_directory_worker_threads(files.len(), options.concurrency);
            self.output
                .found_directory(files.len(), &normalized_path, worker_threads);
            fs::create_dir_all(&directory_target).map_err(|source| AppError::Io {
                path: directory_target.clone(),
                source,
            })?;
            self.output.created_directory(&directory_target);
            let jobs =
                self.build_directory_download_jobs(&directory_target, &normalized_path, files)?;
            self.download_directory_files(client, options, stats, jobs)?;
            return Ok(directory_target);
        }

        Err(AppError::UnexpectedApiResponse)
    }

    fn collect_directory_files(
        &self,
        client: &Client,
        options: &ResolvedOptions,
        stats: &mut DownloadStats,
        remote_path: &str,
        entries: Option<Vec<ContentItem>>,
    ) -> Result<Vec<ContentItem>, AppError> {
        let directory_entries = match entries {
            Some(entries) => entries,
            None => self.get_directory_listing(client, options, remote_path)?,
        };

        let mut files = Vec::new();
        for item in directory_entries {
            let item_path = match item.path.clone() {
                Some(path) => path,
                None => continue,
            };
            match item.kind.as_deref() {
                Some("dir") => files.extend(
                    self.collect_directory_files(client, options, stats, &item_path, None)?,
                ),
                Some("file") => files.push(item),
                other => {
                    stats.skipped_entries += 1;
                    self.output.warning(&format!(
                        "{} {} ({})",
                        match options.language {
                            Language::Zh => "跳过不支持的条目：",
                            Language::En => "Skipping unsupported entry:",
                        },
                        item_path,
                        other.unwrap_or("unknown"),
                    ));
                }
            }
        }
        Ok(files)
    }

    fn get_directory_listing(
        &self,
        client: &Client,
        options: &ResolvedOptions,
        remote_path: &str,
    ) -> Result<Vec<ContentItem>, AppError> {
        let response = self.get_json(
            client,
            &build_contents_api_url(
                &self.config.api_base,
                &options.repo,
                remote_path,
                options.git_ref.as_deref(),
            ),
            options,
        )?;
        serde_json::from_value(response).map_err(|err| AppError::Json(err.to_string()))
    }

    fn save_file(
        &self,
        client: &Client,
        options: &ResolvedOptions,
        item: &ContentItem,
        local_target: &Path,
        shown_path: &str,
    ) -> Result<SaveFileOutcome, AppError> {
        if !options.overwrite && local_target.is_file() {
            self.output.skipping_existing(shown_path);
            return Ok(SaveFileOutcome::SkippedExisting);
        }

        let item_path = item
            .path
            .as_deref()
            .ok_or(AppError::MissingRepositoryPath)?;

        self.output.downloading(shown_path);

        if let Some(download_url) = item.download_url.as_deref() {
            self.debug_log_url(options, "download-url", download_url);
            if self
                .download_from_raw_url(client, options, local_target, download_url)
                .is_ok()
            {
                return Ok(SaveFileOutcome::Downloaded);
            }

            self.output.warning(match options.language {
                Language::Zh => "文件下载失败，正在改用 GitHub Raw API...",
                Language::En => "File download failed, falling back to GitHub Raw API...",
            });
        }

        self.debug_log_strategy(options, RawDownloadStrategy::RawApi);
        self.stream_contents_api_file(client, options, item_path, local_target)?;
        Ok(SaveFileOutcome::Downloaded)
    }

    fn build_directory_download_jobs(
        &self,
        directory_target: &Path,
        remote_root: &str,
        files: Vec<ContentItem>,
    ) -> Result<Vec<DownloadJob>, AppError> {
        files
            .into_iter()
            .map(|item| {
                let item_path = item.path.clone().ok_or(AppError::MissingRepositoryPath)?;
                let relative_part = relative_item_path(remote_root, &item_path);
                Ok(DownloadJob {
                    item,
                    local_target: directory_target.join(&relative_part),
                    shown_path: relative_part,
                })
            })
            .collect()
    }

    fn download_directory_files(
        &self,
        client: &Client,
        options: &ResolvedOptions,
        stats: &mut DownloadStats,
        jobs: Vec<DownloadJob>,
    ) -> Result<(), AppError> {
        let worker = |job| self.download_job(client, options, job);
        for outcome in run_bounded_work(&jobs, options.concurrency, &worker)? {
            match outcome {
                SaveFileOutcome::Downloaded => stats.files_downloaded += 1,
                SaveFileOutcome::SkippedExisting => stats.skipped_existing_files += 1,
            }
        }
        Ok(())
    }

    fn download_job(
        &self,
        client: &Client,
        options: &ResolvedOptions,
        job: DownloadJob,
    ) -> Result<SaveFileOutcome, AppError> {
        self.save_file(
            client,
            options,
            &job.item,
            &job.local_target,
            &job.shown_path,
        )
    }

    fn stream_contents_api_file(
        &self,
        client: &Client,
        options: &ResolvedOptions,
        remote_path: &str,
        local_target: &Path,
    ) -> Result<(), AppError> {
        let api_url = build_contents_api_url(
            &self.config.api_base,
            &options.repo,
            remote_path,
            options.git_ref.as_deref(),
        );
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
        should_attempt_prefix_proxy(
            options.prefix_mode,
            options.token.is_some(),
            &options.proxy_base,
            error,
        )
    }

    fn download_from_raw_url(
        &self,
        client: &Client,
        options: &ResolvedOptions,
        local_target: &Path,
        download_url: &str,
    ) -> Result<(), AppError> {
        match options.prefix_mode {
            PrefixProxyMode::Direct => {
                self.debug_log_strategy(options, RawDownloadStrategy::DirectUrl);
                self.stream_download(client, download_url, local_target, None, None)
            }
            PrefixProxyMode::Fallback => {
                self.debug_log_strategy(options, RawDownloadStrategy::DirectUrl);
                match self.stream_download(client, download_url, local_target, None, None) {
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
                        self.stream_download(client, &proxied_url, local_target, None, None)
                    }
                    Err(error) => Err(error),
                }
            }
            PrefixProxyMode::Prefer => {
                if options.proxy_base.trim().is_empty() {
                    self.debug_log_strategy(options, RawDownloadStrategy::DirectUrl);
                    return self.stream_download(client, download_url, local_target, None, None);
                }

                let proxied_url = join_proxy_url(&options.proxy_base, download_url);
                self.debug_log_strategy(options, RawDownloadStrategy::PrefixProxy);
                self.debug_log_url(options, "prefix-url", &proxied_url);
                match self.stream_download(client, &proxied_url, local_target, None, None) {
                    Ok(()) => Ok(()),
                    Err(_) => {
                        self.output.warning(match options.language {
                            Language::Zh => "前缀代理下载失败，正在回退直连文件 URL...",
                            Language::En => {
                                "Prefix proxy download failed, falling back to direct file URL..."
                            }
                        });
                        self.debug_log_strategy(options, RawDownloadStrategy::DirectUrl);
                        self.stream_download(client, download_url, local_target, None, None)
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

fn run_bounded_work<T, R, E, F>(items: &[T], concurrency: usize, worker: &F) -> Result<Vec<R>, E>
where
    T: Clone + Send,
    R: Send,
    E: Send,
    F: Fn(T) -> Result<R, E> + Sync,
{
    if items.is_empty() {
        return Ok(Vec::new());
    }

    let concurrency = concurrency.max(1);
    let mut results = Vec::with_capacity(items.len());

    for batch in items.chunks(concurrency) {
        let mut first_error = None;
        let mut batch_results = Vec::with_capacity(batch.len());

        thread::scope(|scope| {
            let mut handles = Vec::with_capacity(batch.len());
            for item in batch.iter().cloned() {
                handles.push(scope.spawn(move || worker(item)));
            }

            for handle in handles {
                match handle.join().expect("worker thread panicked") {
                    Ok(result) => batch_results.push(result),
                    Err(error) => {
                        if first_error.is_none() {
                            first_error = Some(error);
                        }
                    }
                }
            }
        });

        if let Some(error) = first_error {
            return Err(error);
        }

        results.extend(batch_results);
    }

    Ok(results)
}

fn effective_directory_worker_threads(file_count: usize, concurrency: usize) -> usize {
    file_count.min(concurrency.max(1))
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

    #[test]
    fn downloads_single_file() {
        let mut server = Server::new();
        let download_url = format!("{}/downloads/README.md", server.url());

        let _metadata = server
            .mock("GET", "/repos/owner/repo/contents/README.md")
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
            git_ref: None,
            token: None,
            api_base: server.url(),
            proxy_base: DEFAULT_GH_PROXY.to_string(),
            prefix_mode: PrefixProxyMode::Fallback,
            concurrency: 4,
            language: Language::En,
            overwrite: false,
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
        assert_eq!(fs::read_to_string(target).expect("file"), "hello from rust");
    }

    #[test]
    fn downloads_nested_directory() {
        let mut server = Server::new();
        let top_file_url = format!("{}/raw/src/main.rs", server.url());
        let nested_file_url = format!("{}/raw/src/nested/lib.rs", server.url());

        let _top_listing = server
            .mock("GET", "/repos/owner/repo/contents/src")
            .match_header("accept", "application/vnd.github+json")
            .with_status(200)
            .with_body(format!(
                r#"[{{"name":"main.rs","path":"src/main.rs","type":"file","download_url":"{}"}},{{"name":"nested","path":"src/nested","type":"dir","download_url":null}}]"#,
                top_file_url
            ))
            .create();

        let _nested_listing = server
            .mock("GET", "/repos/owner/repo/contents/src/nested")
            .match_header("accept", "application/vnd.github+json")
            .with_status(200)
            .with_body(format!(
                r#"[{{"name":"lib.rs","path":"src/nested/lib.rs","type":"file","download_url":"{}"}}]"#,
                nested_file_url
            ))
            .create();

        let _top_file = server
            .mock("GET", "/raw/src/main.rs")
            .with_status(200)
            .with_body("fn main() {}")
            .create();

        let _nested_file = server
            .mock("GET", "/raw/src/nested/lib.rs")
            .with_status(200)
            .with_body("pub fn helper() {}")
            .create();

        let dir = tempdir().expect("tempdir");
        let options = ResolvedOptions {
            repo: "owner/repo".to_string(),
            remote_path: "src".to_string(),
            local_target: dir.path().join("downloads"),
            git_ref: None,
            token: None,
            api_base: server.url(),
            proxy_base: DEFAULT_GH_PROXY.to_string(),
            prefix_mode: PrefixProxyMode::Fallback,
            concurrency: 4,
            language: Language::En,
            overwrite: false,
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
    fn does_not_proxy_github_metadata_api_errors() {
        let mut api_server = Server::new();

        let _direct = api_server
            .mock("GET", "/repos/owner/repo/contents/README.md")
            .match_header("accept", "application/vnd.github+json")
            .with_status(403)
            .with_body(r#"{"message":"rate limited"}"#)
            .create();

        let dir = tempdir().expect("tempdir");
        let options = ResolvedOptions {
            repo: "owner/repo".to_string(),
            remote_path: "README.md".to_string(),
            local_target: dir.path().join("README.md"),
            git_ref: None,
            token: None,
            api_base: api_server.url(),
            proxy_base: DEFAULT_GH_PROXY.to_string(),
            prefix_mode: PrefixProxyMode::Fallback,
            concurrency: 4,
            language: Language::En,
            overwrite: false,
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
            git_ref: None,
            token: None,
            api_base: api_server.url(),
            proxy_base: proxy_server.url(),
            prefix_mode: PrefixProxyMode::Fallback,
            concurrency: 4,
            language: Language::En,
            overwrite: false,
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
    fn prefer_mode_uses_prefix_proxy_first() {
        let mut api_server = Server::new();
        let mut proxy_server = Server::new();
        let mut raw_server = Server::new();

        let download_url = format!("{}/owner/repo/main/README.md", raw_server.url());

        let _metadata = api_server
            .mock("GET", "/repos/owner/repo/contents/README.md")
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
            git_ref: None,
            token: None,
            api_base: api_server.url(),
            proxy_base: proxy_server.url(),
            prefix_mode: PrefixProxyMode::Prefer,
            concurrency: 4,
            language: Language::En,
            overwrite: false,
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
    fn bounded_work_uses_requested_concurrency() {
        let in_flight = Arc::new(AtomicUsize::new(0));
        let max_in_flight = Arc::new(AtomicUsize::new(0));
        let jobs: Vec<_> = (0..6).collect();

        let results = run_bounded_work(&jobs, 2, &|_| {
            let current = in_flight.fetch_add(1, Ordering::SeqCst) + 1;
            max_in_flight.fetch_max(current, Ordering::SeqCst);
            thread::sleep(Duration::from_millis(20));
            in_flight.fetch_sub(1, Ordering::SeqCst);
            Ok::<(), AppError>(())
        })
        .expect("work should succeed");

        assert_eq!(results.len(), 6);
        assert_eq!(max_in_flight.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn bounded_work_returns_error_when_any_job_fails() {
        let jobs: Vec<_> = (0..5).collect();

        let error = run_bounded_work(&jobs, 3, &|job| {
            if job == 3 {
                Err(AppError::InvalidPath("boom".to_string()))
            } else {
                Ok(())
            }
        })
        .expect_err("one failing job should fail the batch");

        assert!(matches!(error, AppError::InvalidPath(message) if message == "boom"));
    }

    #[test]
    fn effective_directory_worker_threads_uses_file_count_and_configured_bound() {
        assert_eq!(effective_directory_worker_threads(11, 4), 4);
        assert_eq!(effective_directory_worker_threads(3, 8), 3);
        assert_eq!(effective_directory_worker_threads(0, 4), 0);
    }

    #[test]
    fn build_directory_download_jobs_preserves_relative_paths() {
        let runner = Runner::new(RuntimeConfig::default(), Output::new(false, Language::En));
        let directory_target = Path::new("/tmp/downloads/src");
        let files = vec![
            ContentItem {
                name: Some("main.rs".to_string()),
                path: Some("src/main.rs".to_string()),
                kind: Some("file".to_string()),
                download_url: Some("https://example.invalid/main.rs".to_string()),
            },
            ContentItem {
                name: Some("lib.rs".to_string()),
                path: Some("src/nested/lib.rs".to_string()),
                kind: Some("file".to_string()),
                download_url: Some("https://example.invalid/nested/lib.rs".to_string()),
            },
        ];

        let jobs = runner
            .build_directory_download_jobs(directory_target, "src", files)
            .expect("jobs should build");

        assert_eq!(jobs.len(), 2);
        assert_eq!(jobs[0].shown_path, "main.rs");
        assert_eq!(jobs[0].local_target, directory_target.join("main.rs"));
        assert_eq!(jobs[1].shown_path, "nested/lib.rs");
        assert_eq!(jobs[1].local_target, directory_target.join("nested/lib.rs"));
    }

    #[test]
    fn skips_existing_direct_file_by_default() {
        let mut server = Server::new();
        let download_url = format!("{}/downloads/README.md", server.url());

        let _metadata = server
            .mock("GET", "/repos/owner/repo/contents/README.md")
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
            git_ref: None,
            token: None,
            api_base: server.url(),
            proxy_base: DEFAULT_GH_PROXY.to_string(),
            prefix_mode: PrefixProxyMode::Fallback,
            concurrency: 4,
            language: Language::En,
            overwrite: false,
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
        let download_url = format!("{}/downloads/README.md", server.url());

        let _metadata = server
            .mock("GET", "/repos/owner/repo/contents/README.md")
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
            git_ref: None,
            token: None,
            api_base: server.url(),
            proxy_base: DEFAULT_GH_PROXY.to_string(),
            prefix_mode: PrefixProxyMode::Fallback,
            concurrency: 4,
            language: Language::En,
            overwrite: true,
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
        let existing_url = format!("{}/raw/src/existing.rs", server.url());
        let new_url = format!("{}/raw/src/new.rs", server.url());

        let _listing = server
            .mock("GET", "/repos/owner/repo/contents/src")
            .match_header("accept", "application/vnd.github+json")
            .with_status(200)
            .with_body(format!(
                r#"[{{"name":"existing.rs","path":"src/existing.rs","type":"file","download_url":"{}"}},{{"name":"new.rs","path":"src/new.rs","type":"file","download_url":"{}"}}]"#,
                existing_url, new_url
            ))
            .create();

        let existing_download = server
            .mock("GET", "/raw/src/existing.rs")
            .expect(0)
            .with_status(200)
            .with_body("should not be downloaded")
            .create();

        let new_download = server
            .mock("GET", "/raw/src/new.rs")
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
            git_ref: None,
            token: None,
            api_base: server.url(),
            proxy_base: DEFAULT_GH_PROXY.to_string(),
            prefix_mode: PrefixProxyMode::Fallback,
            concurrency: 4,
            language: Language::En,
            overwrite: false,
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
        let existing_url = format!("{}/raw/src/existing.rs", server.url());
        let new_url = format!("{}/raw/src/new.rs", server.url());

        let _listing = server
            .mock("GET", "/repos/owner/repo/contents/src")
            .match_header("accept", "application/vnd.github+json")
            .with_status(200)
            .with_body(format!(
                r#"[{{"name":"existing.rs","path":"src/existing.rs","type":"file","download_url":"{}"}},{{"name":"new.rs","path":"src/new.rs","type":"file","download_url":"{}"}}]"#,
                existing_url, new_url
            ))
            .create();

        let existing_download = server
            .mock("GET", "/raw/src/existing.rs")
            .expect(1)
            .with_status(200)
            .with_body("pub fn replaced() {}")
            .create();

        let new_download = server
            .mock("GET", "/raw/src/new.rs")
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
            git_ref: None,
            token: None,
            api_base: server.url(),
            proxy_base: DEFAULT_GH_PROXY.to_string(),
            prefix_mode: PrefixProxyMode::Fallback,
            concurrency: 4,
            language: Language::En,
            overwrite: true,
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
            git_ref: None,
            token: None,
            api_base: format!("{}/enterprise/api/v3/", api_server.url()),
            proxy_base: DEFAULT_GH_PROXY.to_string(),
            prefix_mode: PrefixProxyMode::Direct,
            concurrency: 4,
            language: Language::En,
            overwrite: false,
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
