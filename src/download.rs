use std::fs::{self, File};
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::time::Duration;

use reqwest::blocking::{Client, Response};
use reqwest::header::{ACCEPT, AUTHORIZATION, USER_AGENT};
use serde::Deserialize;
use serde_json::Value;

use crate::cli::{PrefixProxyMode, ResolvedOptions};
use crate::error::AppError;
use crate::i18n::Language;
use crate::output::Output;

pub const DEFAULT_GH_PROXY: &str = "https://gh-proxy.com/";
pub const DEFAULT_GITHUB_API_BASE: &str = "https://api.github.com";
const USER_AGENT_VALUE: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

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
    pub skipped_entries: usize,
}

#[derive(Debug)]
pub struct RunOutcome {
    pub saved_path: PathBuf,
    pub stats: DownloadStats,
}

#[derive(Debug, Deserialize, Clone)]
struct ContentItem {
    name: Option<String>,
    path: Option<String>,
    #[serde(rename = "type")]
    kind: Option<String>,
    download_url: Option<String>,
}

pub struct Runner {
    config: RuntimeConfig,
    output: Output,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RawDownloadStrategy {
    DirectUrl,
    PrefixProxy,
    RawApi,
}

impl RawDownloadStrategy {
    fn as_str(self) -> &'static str {
        match self {
            Self::DirectUrl => "direct-url",
            Self::PrefixProxy => "prefix-proxy",
            Self::RawApi => "raw-api",
        }
    }
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
            self.save_file(client, options, stats, &item, &final_target, &shown_path)?;
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
            self.output.found_directory(files.len(), &normalized_path);
            fs::create_dir_all(&directory_target).map_err(|source| AppError::Io {
                path: directory_target.clone(),
                source,
            })?;
            self.output.created_directory(&directory_target);
            for item in files {
                let item_path = item.path.clone().ok_or(AppError::MissingRepositoryPath)?;
                let relative_part = relative_item_path(&normalized_path, &item_path);
                let local_path = directory_target.join(&relative_part);
                self.save_file(client, options, stats, &item, &local_path, &relative_part)?;
            }
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
        stats: &mut DownloadStats,
        item: &ContentItem,
        local_target: &Path,
        shown_path: &str,
    ) -> Result<(), AppError> {
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
                stats.files_downloaded += 1;
                return Ok(());
            }

            self.output.warning(match options.language {
                Language::Zh => "文件下载失败，正在改用 GitHub Raw API...",
                Language::En => "File download failed, falling back to GitHub Raw API...",
            });
        }

        self.debug_log_strategy(options, RawDownloadStrategy::RawApi);
        self.stream_contents_api_file(client, options, item_path, local_target)?;
        stats.files_downloaded += 1;
        Ok(())
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
        if options.token.is_some() || options.proxy_base.trim().is_empty() {
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

        eprintln!("[debug] {}: {}", label, redact_url_for_display(url));
    }

    fn debug_log_strategy(&self, options: &ResolvedOptions, strategy: RawDownloadStrategy) {
        if !options.debug {
            return;
        }

        eprintln!("[debug] raw-download-strategy: {}", strategy.as_str());
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
        let response = send_request(client, url, accept, token)?;
        response
            .json::<Value>()
            .map_err(|err| AppError::Json(err.to_string()))
    }

    fn stream_download(
        &self,
        client: &Client,
        url: &str,
        local_target: &Path,
        accept: Option<&str>,
        token: Option<&str>,
    ) -> Result<(), AppError> {
        if let Some(parent) = local_target.parent() {
            fs::create_dir_all(parent).map_err(|source| AppError::Io {
                path: parent.to_path_buf(),
                source,
            })?;
        }

        let mut response = send_request(client, url, accept, token)?;
        let mut file = File::create(local_target).map_err(|source| AppError::Io {
            path: local_target.to_path_buf(),
            source,
        })?;
        io::copy(&mut response, &mut file).map_err(|source| AppError::Io {
            path: local_target.to_path_buf(),
            source,
        })?;
        Ok(())
    }
}

fn build_client() -> Result<Client, AppError> {
    Client::builder()
        .timeout(Duration::from_secs(120))
        .connect_timeout(Duration::from_secs(30))
        .redirect(reqwest::redirect::Policy::limited(10))
        .no_proxy()
        .build()
        .map_err(|err| AppError::Request {
            url: None,
            message: err.to_string(),
        })
}

fn send_request(
    client: &Client,
    url: &str,
    accept: Option<&str>,
    token: Option<&str>,
) -> Result<Response, AppError> {
    let mut request = client.get(url).header(USER_AGENT, USER_AGENT_VALUE);
    if let Some(accept) = accept {
        request = request.header(ACCEPT, accept);
    }
    if let Some(token) = token {
        request = request.header(AUTHORIZATION, format!("Bearer {}", token));
    }

    let response = request.send().map_err(|err| AppError::Request {
        url: Some(url.to_string()),
        message: err.to_string(),
    })?;

    let status = response.status();
    if status.is_success() {
        return Ok(response);
    }

    let detail = extract_response_detail(response);
    Err(AppError::HttpStatus {
        status: status.as_u16(),
        url: url.to_string(),
        detail,
    })
}

fn extract_response_detail(mut response: Response) -> Option<String> {
    let mut body = String::new();
    if response.read_to_string(&mut body).is_err() {
        return None;
    }
    if body.trim().is_empty() {
        return None;
    }
    if let Ok(value) = serde_json::from_str::<Value>(&body) {
        return value
            .get("message")
            .and_then(Value::as_str)
            .map(ToOwned::to_owned)
            .or(Some(body));
    }
    Some(body)
}

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

fn choose_file_target(local_target: &Path, remote_path: &str, item: &ContentItem) -> PathBuf {
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

fn file_name_from_remote_path(remote_path: &str) -> String {
    Path::new(remote_path)
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("downloaded-file")
        .to_string()
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::net::TcpListener;
    use std::process::Command;

    use mockito::{Matcher, Server};
    use tempfile::tempdir;

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
    fn user_agent_tracks_package_version() {
        assert_eq!(
            USER_AGENT_VALUE,
            concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"))
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

    #[test]
    fn build_client_ignores_http_proxy_environment() {
        let status = Command::new(std::env::current_exe().expect("current test binary"))
            .arg("download::tests::build_client_ignores_http_proxy_environment_subprocess")
            .arg("--exact")
            .arg("--nocapture")
            .env("GH_DOWNLOAD_RUN_PROXY_SUBPROCESS", "1")
            .status()
            .expect("run proxy subprocess test");

        assert!(status.success(), "proxy subprocess test should pass");
    }

    #[test]
    fn build_client_ignores_http_proxy_environment_subprocess() {
        if std::env::var("GH_DOWNLOAD_RUN_PROXY_SUBPROCESS").as_deref() != Ok("1") {
            return;
        }

        let _env_snapshot = ProxyEnvSnapshot::capture();
        clear_proxy_env();

        let listener = TcpListener::bind("127.0.0.1:0").expect("bind proxy listener");
        let proxy_url = format!("http://{}", listener.local_addr().expect("listener addr"));
        let target_url = "http://not-used.invalid/proxied";

        set_env_var("HTTP_PROXY", Some(&proxy_url));

        let client = build_client().expect("client should build");
        let error = send_request(&client, target_url, None, None)
            .expect_err("request should ignore HTTP_PROXY and fail to resolve target directly");

        match error {
            AppError::Request { message, .. } => {
                assert!(
                    !message.contains(&proxy_url),
                    "request unexpectedly referenced configured proxy"
                );
            }
            other => panic!("expected request error, got {other:?}"),
        }

        assert!(proxy_url.starts_with("http://127.0.0.1:"));
    }

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
            proxy_base: DEFAULT_GH_PROXY.to_string(),
            prefix_mode: PrefixProxyMode::Fallback,
            language: Language::En,
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
            proxy_base: DEFAULT_GH_PROXY.to_string(),
            prefix_mode: PrefixProxyMode::Fallback,
            language: Language::En,
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
            proxy_base: DEFAULT_GH_PROXY.to_string(),
            prefix_mode: PrefixProxyMode::Fallback,
            language: Language::En,
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
            proxy_base: proxy_server.url(),
            prefix_mode: PrefixProxyMode::Fallback,
            language: Language::En,
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
            proxy_base: proxy_server.url(),
            prefix_mode: PrefixProxyMode::Prefer,
            language: Language::En,
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

    fn clear_proxy_env() {
        for key in proxy_env_keys() {
            set_env_var(key, None);
        }
    }

    fn proxy_env_keys() -> &'static [&'static str] {
        &[
            "HTTP_PROXY",
            "http_proxy",
            "HTTPS_PROXY",
            "https_proxy",
            "ALL_PROXY",
            "all_proxy",
            "NO_PROXY",
            "no_proxy",
        ]
    }

    fn set_env_var(key: &str, value: Option<&str>) {
        match value {
            Some(value) => unsafe { std::env::set_var(key, value) },
            None => unsafe { std::env::remove_var(key) },
        }
    }

    #[derive(Debug)]
    struct ProxyEnvSnapshot(Vec<(&'static str, Option<String>)>);

    impl ProxyEnvSnapshot {
        fn capture() -> Self {
            Self(
                proxy_env_keys()
                    .iter()
                    .map(|key| (*key, std::env::var(key).ok()))
                    .collect(),
            )
        }
    }

    impl Drop for ProxyEnvSnapshot {
        fn drop(&mut self) {
            for (key, value) in &self.0 {
                set_env_var(key, value.as_deref());
            }
        }
    }
}
