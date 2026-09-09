fn main() {
    #[cfg(windows)]
    {
        let mut res = winres::WindowsResource::new();
        res.set_manifest_file("app.manifest");
        // Ignore error if winres is optional in dev, or proceed
        let _ = res.compile();
    }
    tauri_build::build()
}
