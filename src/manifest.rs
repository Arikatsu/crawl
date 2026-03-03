use crate::utils;
use serde_json::Value;

pub async fn fetch_manifest_icon(
    client: &reqwest::Client,
    manifest_url: &str,
    base_url: &str,
) -> Option<String> {
    let rsp = client.get(manifest_url).send().await.ok()?;
    if !rsp.status().is_success() {
        return None;
    }

    let text = rsp.text().await.ok()?;
    let manifest: Value = serde_json::from_str(&text).ok()?;
    let icons = manifest.get("icons")?.as_array()?;

    let mut best_url = String::new();
    let mut max_size = 0;

    for icon in icons {
        let src = icon.get("src").and_then(|v| v.as_str()).unwrap_or("");
        if src.is_empty() {
            continue;
        }

        let size = icon.get("sizes")
            .and_then(|v| v.as_str())
            .map(utils::parse_max_icon_size)
            .unwrap_or(0);

        if size >= max_size {
            max_size = size;
            best_url = src.to_string();
        }
    }

    if best_url.is_empty() {
        None
    } else {
        Some(utils::normalize_url(base_url, &best_url))
    }
}