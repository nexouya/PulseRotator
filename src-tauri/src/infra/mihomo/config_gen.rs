// ============================================================================
// File: src-tauri/src/infra/mihomo/config_gen.rs
// Purpose: Dynamic Lean YAML Config Generator for Mihomo Core with TUN
// ============================================================================

use serde_yaml::{Mapping, Value};
use std::path::Path;
use crate::domain::error::AppError;

pub struct ConfigGenerator;

impl ConfigGenerator {
    /// Generate a complete, lean config.yaml for Mihomo
    pub fn generate_yaml(
        sub_url: &str,
        enable_tun: bool,
        secret_token: &str,
    ) -> Result<String, AppError> {
        let mut doc = Mapping::new();

        // 1. Basic Ports
        doc.insert(Value::from("mixed-port"), Value::from(7890));
        doc.insert(Value::from("allow-lan"), Value::from(false));
        doc.insert(Value::from("mode"), Value::from("rule"));
        doc.insert(Value::from("log-level"), Value::from("info"));
        doc.insert(Value::from("ipv6"), Value::from(false));

        // 2. External Controller (REST API)
        doc.insert(Value::from("external-controller"), Value::from("127.0.0.1:9090"));
        if !secret_token.is_empty() {
            doc.insert(Value::from("secret"), Value::from(secret_token));
        }

        // 3. DNS Configuration (Fake-IP for zero lookup latency)
        let mut dns = Mapping::new();
        dns.insert(Value::from("enable"), Value::from(true));
        dns.insert(Value::from("enhanced-mode"), Value::from("fake-ip"));
        dns.insert(Value::from("fake-ip-range"), Value::from("198.18.0.1/16"));
        let mut nameservers = Vec::new();
        nameservers.push(Value::from("https://dns.cloudflare.com/dns-query"));
        nameservers.push(Value::from("https://dns.google/dns-query"));
        nameservers.push(Value::from("1.1.1.1"));
        dns.insert(Value::from("nameserver"), Value::from(nameservers));
        doc.insert(Value::from("dns"), Value::from(dns));

        // 4. TUN Mode Configuration
        let mut tun = Mapping::new();
        tun.insert(Value::from("enable"), Value::from(enable_tun));
        tun.insert(Value::from("stack"), Value::from("mixed"));
        tun.insert(Value::from("auto-route"), Value::from(true));
        tun.insert(Value::from("auto-detect-interface"), Value::from(true));
        let mut hijack = Vec::new();
        hijack.push(Value::from("any:53"));
        hijack.push(Value::from("tcp://any:53"));
        tun.insert(Value::from("dns-hijack"), Value::from(hijack));
        doc.insert(Value::from("tun"), Value::from(tun));

        // 5. Proxy Providers (Remote Subscription Auto-Management)
        let mut providers = Mapping::new();
        let mut sub_prov = Mapping::new();
        sub_prov.insert(Value::from("type"), Value::from("http"));
        sub_prov.insert(Value::from("url"), Value::from(sub_url));
        sub_prov.insert(Value::from("interval"), Value::from(3600)); // Refresh every 1 hr
        sub_prov.insert(Value::from("path"), Value::from("./sub_provider.yaml"));

        let mut health = Mapping::new();
        health.insert(Value::from("enable"), Value::from(true));
        health.insert(Value::from("interval"), Value::from(300));
        health.insert(Value::from("url"), Value::from("http://cp.cloudflare.com/generate_204"));
        sub_prov.insert(Value::from("health-check"), Value::from(health));

        providers.insert(Value::from("default_sub"), Value::from(sub_prov));
        doc.insert(Value::from("proxy-providers"), Value::from(providers));

        // 6. Proxy Groups (ROTATOR select group)
        let mut groups = Vec::new();

        let mut rotator_group = Mapping::new();
        rotator_group.insert(Value::from("name"), Value::from("ROTATOR"));
        rotator_group.insert(Value::from("type"), Value::from("select"));
        let mut use_list = Vec::new();
        use_list.push(Value::from("default_sub"));
        rotator_group.insert(Value::from("use"), Value::from(use_list));
        groups.push(Value::from(rotator_group));

        doc.insert(Value::from("proxy-groups"), Value::from(groups));

        // 7. Routing Rules
        let mut rules = Vec::new();
        rules.push(Value::from("MATCH,ROTATOR"));
        doc.insert(Value::from("rules"), Value::from(rules));

        serde_yaml::to_string(&doc).map_err(|e| AppError::Generic(e.to_string()))
    }

    /// Save generated yaml to disk
    pub fn save_to_file(path: &Path, content: &str) -> Result<(), AppError> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, content)?;
        Ok(())
    }
}
