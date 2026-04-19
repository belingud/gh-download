use std::fs::{self, File};
use std::io::{self, Read};
use std::path::Path;
use std::time::Duration;

use reqwest::blocking::{Client, Response};
use reqwest::header::{ACCEPT, AUTHORIZATION, USER_AGENT};
use serde_json::Value;

use crate::error::AppError;

pub(super) const USER_AGENT_VALUE: &str =
    concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

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

    use super::*;

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
