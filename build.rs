//! Surface enabled Cargo features in `--version` so installs are easy to verify.

fn main() {
    let pkg = std::env::var("CARGO_PKG_VERSION").unwrap_or_else(|_| "0.0.0".into());
    let mut feats = Vec::new();
    if std::env::var_os("CARGO_FEATURE_MCP").is_some() {
        feats.push("mcp");
    }
    if std::env::var_os("CARGO_FEATURE_CLI").is_some() {
        feats.push("cli");
    }
    let f = if feats.is_empty() {
        "none".to_string()
    } else {
        feats.join(",")
    };
    println!("cargo:rustc-env=JOICY_CLI_VERSION={pkg} [features: {f}]");
}
