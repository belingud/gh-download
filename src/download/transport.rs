use std::ffi::{OsStr, OsString};
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::process;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use reqwest::blocking::{Client, Response};
use reqwest::header::{ACCEPT, AUTHORIZATION, USER_AGENT};
use serde_json::Value;

use crate::error::AppError;

pub(super) const USER_AGENT_VALUE: &str =
    concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

static TEMP_FILE_COUNTER: AtomicU64 = AtomicU64::new(0);

pub(super) fn build_client() -> Result<Client, AppError> {
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

pub(super) fn send_request(
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

pub(super) fn send_json_request(
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

pub(super) fn stream_download(
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

    let response = send_request(client, url, accept, token)?;
    copy_reader_to_target_atomically(response, local_target)
}

fn copy_reader_to_target_atomically<R: Read>(
    mut reader: R,
    local_target: &Path,
) -> Result<(), AppError> {
    if let Some(parent) = local_target.parent() {
        fs::create_dir_all(parent).map_err(|source| AppError::Io {
            path: parent.to_path_buf(),
            source,
        })?;
    }

    let (temporary_target, mut file) = create_temporary_file(local_target)?;
    if let Err(source) = io::copy(&mut reader, &mut file) {
        drop(file);
        let _ = fs::remove_file(&temporary_target);
        return Err(AppError::Io {
            path: local_target.to_path_buf(),
            source,
        });
    }
    drop(file);

    commit_temporary_file(&temporary_target, local_target).map_err(|source| {
        let _ = fs::remove_file(&temporary_target);
        AppError::Io {
            path: local_target.to_path_buf(),
            source,
        }
    })?;

    Ok(())
}

fn create_temporary_file(local_target: &Path) -> Result<(PathBuf, File), AppError> {
    for _ in 0..16 {
        let temporary_target = temporary_sibling_path(local_target, "tmp");
        match OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary_target)
        {
            Ok(file) => return Ok((temporary_target, file)),
            Err(source) if source.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(source) => {
                return Err(AppError::Io {
                    path: local_target.to_path_buf(),
                    source,
                });
            }
        }
    }

    Err(AppError::Io {
        path: local_target.to_path_buf(),
        source: io::Error::new(
            io::ErrorKind::AlreadyExists,
            "could not create a unique temporary download file",
        ),
    })
}

fn commit_temporary_file(temporary_target: &Path, local_target: &Path) -> io::Result<()> {
    match fs::rename(temporary_target, local_target) {
        Ok(()) => Ok(()),
        Err(source) if source.kind() == io::ErrorKind::AlreadyExists && local_target.is_file() => {
            replace_existing_file(temporary_target, local_target)
        }
        Err(source) => Err(source),
    }
}

fn replace_existing_file(temporary_target: &Path, local_target: &Path) -> io::Result<()> {
    let backup_target = move_existing_file_to_backup(local_target)?;
    match fs::rename(temporary_target, local_target) {
        Ok(()) => {
            let _ = fs::remove_file(backup_target);
            Ok(())
        }
        Err(source) => {
            let _ = fs::rename(&backup_target, local_target);
            Err(source)
        }
    }
}

fn move_existing_file_to_backup(local_target: &Path) -> io::Result<PathBuf> {
    for _ in 0..16 {
        let backup_target = temporary_sibling_path(local_target, "bak");
        if backup_target.exists() {
            continue;
        }
        match fs::rename(local_target, &backup_target) {
            Ok(()) => return Ok(backup_target),
            Err(source) if source.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(source) => return Err(source),
        }
    }

    Err(io::Error::new(
        io::ErrorKind::AlreadyExists,
        "could not create a unique backup download file",
    ))
}

fn temporary_sibling_path(local_target: &Path, suffix: &str) -> PathBuf {
    let parent = local_target
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let file_name = local_target
        .file_name()
        .unwrap_or_else(|| OsStr::new("download"));
    let counter = TEMP_FILE_COUNTER.fetch_add(1, Ordering::Relaxed);

    let mut temporary_name = OsString::from(".");
    temporary_name.push(file_name);
    temporary_name.push(format!(
        ".gh-download-{}-{}.{}",
        process::id(),
        counter,
        suffix
    ));
    parent.join(temporary_name)
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

#[cfg(test)]
mod tests {
    use std::net::TcpListener;
    use std::process::Command;

    use tempfile::tempdir;

    use super::*;

    #[test]
    fn atomic_copy_commits_new_file_on_success() {
        let dir = tempdir().expect("tempdir");
        let target = dir.path().join("README.md");

        copy_reader_to_target_atomically("new body".as_bytes(), &target)
            .expect("copy should succeed");

        assert_eq!(fs::read_to_string(target).expect("target"), "new body");
    }

    #[test]
    fn atomic_copy_replaces_existing_file_on_success() {
        let dir = tempdir().expect("tempdir");
        let target = dir.path().join("README.md");
        fs::write(&target, "old body").expect("seed target");

        copy_reader_to_target_atomically("new body".as_bytes(), &target)
            .expect("copy should succeed");

        assert_eq!(fs::read_to_string(target).expect("target"), "new body");
    }

    #[test]
    fn failed_atomic_copy_does_not_create_final_target() {
        let dir = tempdir().expect("tempdir");
        let target = dir.path().join("README.md");

        copy_reader_to_target_atomically(FailingReader::new("partial"), &target)
            .expect_err("copy should fail");

        assert!(!target.exists(), "target should not be created");
        assert!(
            fs::read_dir(dir.path())
                .expect("read temp dir")
                .next()
                .is_none(),
            "temporary file should be removed"
        );
    }

    #[test]
    fn failed_atomic_copy_keeps_existing_final_target() {
        let dir = tempdir().expect("tempdir");
        let target = dir.path().join("README.md");
        fs::write(&target, "old body").expect("seed target");

        copy_reader_to_target_atomically(FailingReader::new("partial"), &target)
            .expect_err("copy should fail");

        assert_eq!(fs::read_to_string(&target).expect("target"), "old body");
        let entries = fs::read_dir(dir.path())
            .expect("read temp dir")
            .collect::<Result<Vec<_>, _>>()
            .expect("dir entries");
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].path(), target);
    }

    struct FailingReader {
        body: &'static [u8],
        has_sent_body: bool,
    }

    impl FailingReader {
        fn new(body: &'static str) -> Self {
            Self {
                body: body.as_bytes(),
                has_sent_body: false,
            }
        }
    }

    impl Read for FailingReader {
        fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
            if self.has_sent_body {
                return Err(io::Error::other("simulated stream failure"));
            }

            self.has_sent_body = true;
            let bytes_to_copy = self.body.len().min(buffer.len());
            buffer[..bytes_to_copy].copy_from_slice(&self.body[..bytes_to_copy]);
            Ok(bytes_to_copy)
        }
    }

    #[test]
    fn user_agent_tracks_package_version() {
        assert_eq!(
            USER_AGENT_VALUE,
            concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"))
        );
    }

    #[test]
    fn build_client_ignores_http_proxy_environment() {
        let status = Command::new(std::env::current_exe().expect("current test binary"))
            .arg("download::transport::tests::build_client_ignores_http_proxy_environment_subprocess")
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
