# PulseRotator (NexuFlux) ⚡

> **High-Performance, Blazing-Fast Automated IP Rotator & Full-Tunnel Client for Windows & Linux.**

Built with **Tauri 2 + Rust + Mihomo (Clash.Meta) Core + React/TailwindCSS**.

---

## 🌟 Features

- 🔄 **Seamless IP Rotation**: Switches outbound public IP at user-defined intervals (e.g. every 10s, 30s, 60s) via lightweight local REST calls without restarting processes or dropping open sockets.
- 🛡️ **Full Tunnel (TUN Mode)**: Routes 100% of Windows OS traffic (browsers, command-line, games, UDP) through a high-performance virtual adapter using `wintun.dll`.
- 😴 **Smart Quarantine (Circuit Breaker)**: Unstable/dead nodes are temporarily put to sleep (10 minutes) instead of being permanently dropped. Background revival loops re-test and bring them back automatically.
- 🚀 **Hexagonal Clean Architecture**: Pure domain models and trait abstractions (`CoreController`) make the codebase super modular and ready to expand into a standalone proxy client anytime.
- 🔒 **Zero Zombie Processes**: Integrates with Windows Job Objects (`JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE`) to guarantee that child core processes terminate instantly if the app is closed or killed.
- 🌐 **Massive Subscription Support**: Streaming parser handles huge VLESS / VMess / Reality / Shadowsocks / Trojan subscription lists without UI lag.

---

## 🏗️ Architecture Overview

```text
src-tauri/src/
├── domain/              # Pure business logic, models, errors, CoreController trait
│   ├── model.rs
│   ├── error.rs
│   └── traits.rs
├── infra/               # Hardware, OS, Mihomo REST API, Job Objects
│   ├── mihomo/          # REST client (127.0.0.1:9090) & dynamic YAML generator
│   ├── process/         # Windows Job Object & Sidecar process manager
│   └── system/          # Admin check, DNS flush (ipconfig /flushdns)
├── app/                 # Application services
│   ├── rotator.rs       # Async countdown loop & IP switcher
│   ├── quarantine.rs    # Thread-safe node pool with sleep & auto-revival
│   └── sub_parser.rs    # Streaming Base64 and URI parser
└── commands.rs          # Tauri IPC commands
```

---

## 🚀 How to Build on GitHub Actions (Zero Local CPU/RAM usage)

A complete automated CI/CD workflow is provided at `.github/workflows/release.yml`.

1. Initialize Git in the project:
   ```bash
   git init
   git add .
   git commit -m "feat: initial PulseRotator implementation"
   ```
2. Create a repository on GitHub (e.g., `PulseRotator`) and push:
   ```bash
   git remote add origin https://github.com/YOUR_USERNAME/PulseRotator.git
   git branch -M main
   git push -u origin main
   ```
3. Push a tag to automatically trigger a Windows `.exe` and `.msi` release:
   ```bash
   git tag v0.1.0
   git push origin v0.1.0
   ```
4. Download the ready-to-run `.exe` setup file from your GitHub repository's **Releases** tab!
