fn main() {
    println!("cargo:rerun-if-env-changed=TRADEX_ORDER_GATEWAY_SHA256");
    let pinned_gateway = std::env::var("TRADEX_ORDER_GATEWAY_SHA256").unwrap_or_default();
    if !pinned_gateway.is_empty()
        && (pinned_gateway.len() != 64
            || !pinned_gateway
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte)))
    {
        panic!("TRADEX_ORDER_GATEWAY_SHA256 must be a lowercase SHA-256 digest");
    }
    println!("cargo:rustc-env=TRADEX_ORDER_GATEWAY_SHA256={pinned_gateway}");
    if std::env::var_os("CARGO_FEATURE_DESKTOP").is_some() {
        tauri_build::build();
    }
}
