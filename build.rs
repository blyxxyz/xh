use std::env;
use std::fs::read_dir;
use std::path::Path;
use std::process::Command;

use syntect::dumps::dump_to_file;
use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxSetBuilder;

fn build_syntax(dir: &str, out: &str) {
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let mut builder = SyntaxSetBuilder::new();
    builder.add_from_folder(dir, true).unwrap();
    let ss = builder.build();
    dump_to_file(&ss, Path::new(&out_dir).join(out)).unwrap();
}

fn feature_status(feature: &str) -> String {
    if env::var_os(format!(
        "CARGO_FEATURE_{}",
        feature.to_uppercase().replace('-', "_")
    ))
    .is_some()
    {
        format!("+{feature}")
    } else {
        format!("-{feature}")
    }
}

fn features() -> String {
    ["native-tls", "rustls", "http3", "network-interface"]
        .map(feature_status)
        .join(" ")
}

/// Cargo doesn't give us the Rust version directly but it does give us $RUSTC.
///
/// Partially adapted from serde. Should be robust.
fn rustc_version() -> Option<String> {
    let rustc = env::var_os("RUSTC")?;
    let output = Command::new(rustc).arg("--version").output().ok()?;
    let mut version = String::from_utf8(output.stdout).ok()?;
    if !version.contains("nightly") {
        if let Some(idx) = version.find(" (") {
            // "rustc 1.91.1 (ed61e7d7e 2025-11-07)" → "rustc 1.91.1"
            version.truncate(idx);
        }
    }
    Some(version)
}

fn main() {
    for dir in [
        "assets/syntax",
        "assets/syntax/basic",
        "assets/syntax/large",
        "assets/themes",
    ] {
        println!("cargo:rerun-if-changed={dir}");
        for entry in read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            let path = path.to_str().unwrap();
            if path.ends_with(".sublime-syntax") || path.ends_with(".tmTheme") {
                println!("cargo:rerun-if-changed={path}");
            }
        }
    }

    build_syntax("assets/syntax/basic", "basic.packdump");
    build_syntax("assets/syntax/large", "large.packdump");

    let out_dir = env::var_os("OUT_DIR").unwrap();
    let ts = ThemeSet::load_from_folder("assets/themes").unwrap();
    dump_to_file(&ts, Path::new(&out_dir).join("themepack.themedump")).unwrap();

    let version = rustc_version().unwrap_or_else(|| "unknown rustc".into());
    let target = env::var("TARGET").unwrap_or_else(|_| "unknown target".into());

    println!("cargo:rustc-env=XH_FEATURES={}", features());
    println!("cargo:rustc-env=XH_ENVIRONMENT={version} {target}");
}
