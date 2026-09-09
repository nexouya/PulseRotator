// ============================================================================
// File: src-tauri/src/app/sub_parser.rs
// Purpose: High-Throughput Streaming Subscription Parser & Decoder
// Supports: Raw URLs, Base64 batches, VLESS, VMess, Reality, SS, Trojan
// ============================================================================

use base64::prelude::*;
use std::time::Duration;
use tracing::{info, warn};
use url::Url;

use crate::domain::error::AppError;
use crate::domain::model::ProxyNode;

pub struct SubParser;

impl SubParser {
    /// Fetch and parse subscription from a remote URL or direct text
    pub async fn fetch_and_parse(url_or_raw: &str) -> Result<Vec<ProxyNode>, AppError> {
        let raw_content = if url_or_raw.starts_with("http://") || url_or_raw.starts_with("https://") {
            info!("Fetching remote subscription from: {}", url_or_raw);
            let client = reqwest::Client::builder()
                .timeout(Duration::from_secs(12))
                .build()
                .map_err(|e| AppError::Generic(e.to_string()))?;

            let resp = client
                .get(url_or_raw)
                .header("User-Agent", "ClashMeta/1.19.0 (PulseRotator; +https://github.com/MetaCubeX/mihomo)")
                .send()
                .await
                .map_err(|e| AppError::SubParse(format!("HTTP fetch failed: {}", e)))?;

            resp.text().await.map_err(|e| AppError::SubParse(e.to_string()))?
        } else {
            url_or_raw.to_string()
        };

        Self::parse_content(&raw_content)
    }

    /// Parse raw subscription content (decoding Base64 if needed)
    pub fn parse_content(content: &str) -> Result<Vec<ProxyNode>, AppError> {
        let cleaned = content.trim();

        // 1. Try decoding as Base64 if it does not start with typical protocol prefixes
        let decoded_text = if !cleaned.starts_with("vless://")
            && !cleaned.starts_with("vmess://")
            && !cleaned.starts_with("ss://")
            && !cleaned.starts_with("trojan://")
            && !cleaned.starts_with("hysteria2://")
            && !cleaned.starts_with("hy2://")
        {
            // Remove whitespace or newlines inside base64 string
            let stripped: String = cleaned.chars().filter(|c| !c.is_whitespace()).collect();
            if let Ok(bytes) = BASE64_STANDARD.decode(&stripped) {
                String::from_utf8(bytes).unwrap_or_else(|_| cleaned.to_string())
            } else if let Ok(bytes) = BASE64_URL_SAFE.decode(&stripped) {
                String::from_utf8(bytes).unwrap_or_else(|_| cleaned.to_string())
            } else {
                cleaned.to_string()
            }
        } else {
            cleaned.to_string()
        };

        // 2. Extract lines and parse node protocols
        let mut nodes = Vec::new();
        for (i, line) in decoded_text.lines().enumerate() {
            let line_trim = line.trim();
            if line_trim.is_empty() {
                continue;
            }

            if let Some(node) = Self::parse_single_uri(line_trim, i + 1) {
                nodes.push(node);
            }
        }

        info!("Parsed {} proxy nodes from subscription source", nodes.len());
        Ok(nodes)
    }

    /// Parse a single proxy URI link
    fn parse_single_uri(uri: &str, index: usize) -> Option<ProxyNode> {
        if let Ok(parsed_url) = Url::parse(uri) {
            let scheme = parsed_url.scheme().to_lowercase();
            let server = parsed_url.host_str().unwrap_or("").to_string();
            let port = parsed_url.port().unwrap_or(443);

            // Extract fragment name (e.g. #US-Reality-01)
            let raw_name = parsed_url.fragment().unwrap_or("");
            let name = if !raw_name.is_empty() {
                urlencoding::decode(raw_name)
                    .map(|c| c.into_owned())
                    .unwrap_or_else(|_| raw_name.to_string())
            } else {
                format!("{}-node-{}", scheme.to_uppercase(), index)
            };

            match scheme.as_str() {
                "vless" | "vmess" | "ss" | "trojan" | "hysteria2" | "hy2" | "tuic" => {
                    Some(ProxyNode::new(name, scheme.to_uppercase(), server, port))
                }
                _ => None,
            }
        } else {
            None
        }
    }
}
