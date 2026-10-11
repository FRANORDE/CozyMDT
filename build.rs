// Embeds the icon (and version info taken from Cargo.toml) into the .exe
fn main() {
    // Re-run this script whenever the icon changes
    println!("cargo:rerun-if-changed=assets/CozyMDT.ico");

    // Only runs when targeting Windows
    if std::env::var("CARGO_CFG_TARGET_OS").unwrap() == "windows" {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("assets/CozyMDT.ico");
        res.compile().expect("failed to embed the icon");
    }
}
