
// Embeds the icon (and version info taken from Cargo.toml) into the .exe
fn main() {
    // Only runs when targeting Windows
    if std::env::var("CARGO_CFG_TARGET_OS").unwrap() == "windows" {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("assets/CozyMDT.ico");
        res.compile().unwrap();
    }
}
