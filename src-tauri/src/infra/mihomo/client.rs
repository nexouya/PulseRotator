// ============================================================================
// File: src-tauri/src/infra/mihomo/client.rs
// Purpose: Mihomo (Clash.Meta) REST API Driver & CoreController Implementation
// Docs: http://127.0.0.1:9090 (External Controller)
// ============================================================================

use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use tracing::info;

use crate::domain::error::AppError;
use crate::domain::model::{ProxyNode, PublicIpInfo};
use crate::domain::traits::CoreController;
use crate::infra::process::sidecar::SidecarManager;

pub struct MihomoController {
    api_url: String,
    secret: Option<String>,
    http_client: Client,
    sidecar: Arc<SidecarManager>,
}

#[derive(Deserialize, Debug)]
struct MihomoDelayResponse {
    delay: u32,
}

#[derive(Deserialize, Debug)]
struct MihomoProxiesResponse {
    proxies: HashMap<String, MihomoProxyItem>,
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct MihomoProxyItem {
    name: String,
    #[serde(rename = "type")]
    proxy_type: String,
    history: Option<Vec<MihomoDelayHistory>>,
}

#[derive(Deserialize, Debug)]
struct MihomoDelayHistory {
    delay: u32,
}

#[derive(Serialize)]
struct SelectProxyRequest<'a> {
    name: &'a str,
}

impl MihomoController {
    pub fn new(
        api_url: String,
        secret: Option<String>,
        sidecar: Arc<SidecarManager>,
    ) -> Self {
        let http_client = Client::builder()
            .timeout(Duration::from_secs(5))
            .build()
            .unwrap_or_default();

        Self {
            api_url,
            secret,
            http_client,
            sidecar,
        }
    }

    fn auth_header(&self, req: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        if let Some(ref sec) = self.secret {
            if !sec.is_empty() {
                return req.header("Authorization", format!("Bearer {}", sec));
            }
        }
        req
    }
}

#[async_trait]
impl CoreController for MihomoController {
    async fn start(&self, config_path: &Path) -> Result<(), AppError> {
        let work_dir = config_path.parent().unwrap_or_else(|| Path::new("."));
        self.sidecar.spawn(work_dir, config_path)?;

        // Wait up to 5 seconds for REST API to become responsive
        let start_time = std::time::Instant::now();
        loop {
            tokio::time::sleep(Duration::from_millis(200)).await;
            let url = format!("{}/version", self.api_url);
            let req = self.auth_header(self.http_client.get(&url));
            if let Ok(resp) = req.send().await {
                if resp.status().is_success() {
                    info!("Mihomo REST API is healthy and responding!");
                    return Ok(());
                }
            }
            if start_time.elapsed() > Duration::from_secs(6) {
                return Err(AppError::CoreProcess(
                    "Timed out waiting for Mihomo REST API to initialize".to_string(),
                ));
            }
        }
    }

    async fn stop(&self) -> Result<(), AppError> {
        self.sidecar.terminate()
    }

    async fn select_proxy(&self, group: &str, node: &str) -> Result<(), AppError> {
        let url = format!("{}/proxies/{}", self.api_url, urlencoding::encode(group));
        let body = SelectProxyRequest { name: node };

        let req = self.auth_header(self.http_client.put(&url).json(&body));
        let resp = req.send().await.map_err(|e| AppError::Api(e.to_string()))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let txt = resp.text().await.unwrap_or_default();
            return Err(AppError::Api(format!(
                "Failed to switch proxy to '{}': {} - {}",
                node, status, txt
            )));
        }

        info!("Switched group '{}' to proxy '{}'", group, node);
        Ok(())
    }

    async fn test_delay(
        &self,
        node: &str,
        test_url: &str,
        timeout_ms: u32,
    ) -> Result<u32, AppError> {
        let url = format!(
            "{}/proxies/{}/delay?url={}&timeout={}",
            self.api_url,
            urlencoding::encode(node),
            urlencoding::encode(test_url),
            timeout_ms
        );

        let req = self.auth_header(self.http_client.get(&url));
        let resp = req.send().await.map_err(|e| AppError::Api(e.to_string()))?;

        if resp.status().is_success() {
            let data: MihomoDelayResponse = resp.json().await.map_err(|e| AppError::Api(e.to_string()))?;
            Ok(data.delay)
        } else {
            Err(AppError::Api(format!(
                "Delay test failed with status: {}",
                resp.status()
            )))
        }
    }

    async fn query_proxies(&self) -> Result<Vec<ProxyNode>, AppError> {
        let url = format!("{}/proxies", self.api_url);
        let req = self.auth_header(self.http_client.get(&url));
        let resp = req.send().await.map_err(|e| AppError::Api(e.to_string()))?;

        let data: MihomoProxiesResponse = resp.json().await.map_err(|e| AppError::Api(e.to_string()))?;
        let mut list = Vec::new();

        for (name, item) in data.proxies {
            // Filter out system selector groups (DIRECT, REJECT, GLOBAL, ROTATOR, etc.)
            let lower_type = item.proxy_type.to_lowercase();
            if lower_type == "selector"
                || lower_type == "urltest"
                || lower_type == "fallback"
                || lower_type == "loadbalance"
                || lower_type == "direct"
                || lower_type == "reject"
                || lower_type == "compatible"
            {
                continue;
            }

            let latency = item.history.as_ref().and_then(|h| h.last()).map(|d| d.delay);
            let mut node = ProxyNode::new(name, item.proxy_type, "".to_string(), 0);
            if let Some(lat) = latency {
                node.mark_alive(lat);
            }
            list.push(node);
        }

        Ok(list)
    }

    async fn query_public_ip(&self) -> Result<PublicIpInfo, AppError> {
        // Query a lightweight JSON endpoint through local mixed proxy 127.0.0.1:7890
        let proxy = reqwest::Proxy::all("http://127.0.0.1:7890")
            .map_err(|e| AppError::Generic(e.to_string()))?;

        let client = Client::builder()
            .proxy(proxy)
            .timeout(Duration::from_secs(4))
            .build()
            .map_err(|e| AppError::Generic(e.to_string()))?;

        // Fallback endpoints for reliability
        let endpoints = [
            "https://ipapi.co/json/",
            "https://ipwho.is/",
            "http://ip-api.com/json",
        ];

        for ep in endpoints {
            if let Ok(resp) = client.get(ep).send().await {
                if let Ok(json) = resp.json::<serde_json::Value>().await {
                    let ip = json["ip"].as_str().or(json["query"].as_str()).unwrap_or("Unknown").to_string();
                    let country = json["country_name"].as_str().or(json["country"].as_str()).unwrap_or("Unknown").to_string();
                    let country_code = json["country_code"].as_str().or(json["countryCode"].as_str()).unwrap_or("UN").to_string();
                    let city = json["city"].as_str().unwrap_or("").to_string();
                    let org = json["org"].as_str().or(json["isp"].as_str()).unwrap_or("").to_string();

                    return Ok(PublicIpInfo {
                        ip,
                        country,
                        country_code,
                        city,
                        org,
                    });
                }
            }
        }

        Err(AppError::Generic("Failed to query public IP".to_string()))
    }

    async fn is_running(&self) -> bool {
        self.sidecar.is_alive()
    }
}
