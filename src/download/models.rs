use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub(super) struct ContentItem {
    pub(super) name: Option<String>,
    pub(super) path: Option<String>,
    #[serde(rename = "type")]
    #[allow(dead_code)]
    pub(super) kind: Option<String>,
    pub(super) download_url: Option<String>,
}
