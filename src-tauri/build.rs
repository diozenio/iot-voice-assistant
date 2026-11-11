use std::{env, fs, path::PathBuf};

const VARIABLES: [&str; 5] = [
    "OPENROUTER_API_KEY",
    "MODELO_TRANSCRICAO",
    "MODELO_IA",
    "MODELO_VOZ",
    "VOZ",
];

fn main() {
    embed_env();
    tauri_build::build()
}

fn embed_env() {
    let root = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let path = root.join("..").join(".env");
    println!("cargo:rerun-if-changed={}", path.display());

    let Ok(contents) = fs::read_to_string(&path) else {
        return;
    };

    for line in contents.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        let key = key.trim();
        if !VARIABLES.contains(&key) {
            continue;
        }
        let value = value.trim().trim_matches('"').trim_matches('\'');
        println!("cargo:rustc-env=DEFAULT_{key}={value}");
    }
}
