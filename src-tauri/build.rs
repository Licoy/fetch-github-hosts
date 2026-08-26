fn main() {
    // tauri::generate_context! requires frontendDist to exist at compile time.
    // Only create a stub if the real Nuxt output is missing — never overwrite it.
    #[cfg(feature = "gui")]
    ensure_frontend_dist();

    #[cfg(feature = "gui")]
    tauri_build::build();
}

#[cfg(feature = "gui")]
fn ensure_frontend_dist() {
    let dir = std::path::Path::new("../.output/public");
    let index = dir.join("index.html");
    if index.exists() {
        return;
    }
    let _ = std::fs::create_dir_all(dir);
    let _ = std::fs::write(
        index,
        "<!doctype html><html><body></body></html>\n",
    );
}
