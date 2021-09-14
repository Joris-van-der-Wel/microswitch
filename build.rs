extern crate image;
extern crate yaml_rust;
use std::{env, fs};
use std::path::PathBuf;
use yaml_rust::YamlLoader;
use path_absolutize::Absolutize;
use std::collections::HashSet;

fn package_dir() -> String {
    env::var("CARGO_MANIFEST_DIR").expect("No CARGO_MANIFEST_DIR env var")
}

fn out_dir() -> String {
    env::var("OUT_DIR").expect("No OUT_DIR env var")
}

fn rust_string(input: &str) -> String {
    const START: &str = "r####\"";
    const END: &str = "\"####";
    let mut result = String::new();
    result.push_str(START);
    assert!(!input.contains(END));
    result.push_str(input);
    result.push_str(END);
    result
}

const EMPTY_EMBEDDED_CONFIG: &str = r##"
use std::collections::HashMap;

pub fn embedded_samples() -> HashMap<&'static str, &'static [u8]> {
    HashMap::new()
}
pub fn embedded_config() ->  Option<&'static str> {
    None
}
"##;

fn build_embedded_config() {
    let out_file: PathBuf = [&out_dir(), "embedded_config.rs"].iter().collect();
    let config_path = env::var("MICROSWITCH_EMBED_CONFIG_PATH");

    if Err(env::VarError::NotPresent) == config_path {
        println!("DEBUG: writing empty embedded config to {}", out_file.to_str().unwrap());
        fs::write(&out_file, EMPTY_EMBEDDED_CONFIG).expect("failed to write embedded_config.rs");
        return;
    }

    let config_path = config_path.expect("Invalid MICROSWITCH_EMBED_CONFIG_PATH");
    let config_path = PathBuf::from(config_path);
    let config_path = config_path.absolutize().unwrap().into_owned();

    println!("DEBUG: writing embedded config from {} to {}", config_path.to_str().unwrap(), out_file.to_str().unwrap());

    let config_str = fs::read_to_string(&config_path).expect("Failed to read MICROSWITCH_EMBED_CONFIG_PATH");
    let mut resolve_path = config_path;
    resolve_path.pop();

    let mut config = YamlLoader::load_from_str(&config_str).unwrap();
    let config = config.remove(0);

    let mut seen_paths: HashSet<String> = HashSet::new();
    let mut generated_source = String::new();

    generated_source.push_str("use std::collections::HashMap;\n\n");

    generated_source.push_str("pub fn embedded_samples() -> HashMap<&'static str, &'static [u8]> {\n");
    generated_source.push_str("    let items: Vec<(&'static str, &'static [u8])> = vec![\n");

    let banks = config["banks"].as_vec().expect("expected banks to be an array");
    for bank in banks {
        let samples = bank["samples"].as_vec().expect("expected banks[x].samples to be an array");

        for sample in samples {
            let sample_file = sample["file"].as_str().expect("expected banks[x].samples[x].file to be a string");
            let mut sample_file_resolved = PathBuf::from(&resolve_path);
            sample_file_resolved.push(sample_file);

            if seen_paths.contains(sample_file) {
                continue;
            }

            seen_paths.insert(sample_file.to_string());
            let sample_file_resolved = sample_file_resolved.to_str().unwrap();

            generated_source.push_str("        (");
            generated_source.push_str(&rust_string(&sample_file));
            generated_source.push_str(", include_bytes!(");
            generated_source.push_str(&rust_string(sample_file_resolved));
            generated_source.push_str(")),\n");
        }
    }

    generated_source.push_str("    ];\n");
    generated_source.push_str("    items.into_iter().collect()\n");
    generated_source.push_str("}\n");

    generated_source.push_str("pub fn embedded_config() -> Option<&'static str> {\n");
    generated_source.push_str("Some(\n");
    generated_source.push_str(&rust_string(&config_str));
    generated_source.push_str("\n");
    generated_source.push_str(")\n");
    generated_source.push_str("}");

    fs::write(&out_file, &generated_source).expect("failed to write embedded_config.rs");
}

fn build_window_icon() {
    let out_dir = out_dir();
    let img_path: PathBuf = [package_dir().as_str(), "resources", "microswitch-icon-32.png"].iter().collect();
    let out_path: PathBuf = [out_dir.as_str(), "microswitch-icon-32-rgba"].iter().collect();

    let img = image::open(img_path).expect("Failed to read/decode microswitch-icon-32-rgba");
    let img = img.to_rgba8();
    let rgba = img.into_raw();
    println!("DEBUG: writing window icon to {}", out_path.to_str().unwrap());
    fs::write(&out_path, rgba).expect("Failed to write to microswitch-icon-32-rgba");
}

#[cfg(target_os = "windows")]
fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-env-changed=MICROSWITCH_EMBED_CONFIG_PATH");

    let package_dir = package_dir();
    let resources_dir: PathBuf = [package_dir.as_str(), "resources"].iter().collect();

    println!("cargo:rustc-link-search=native={}", resources_dir.to_str().unwrap());
    println!("cargo:rustc-link-lib=dylib={}", "resources");

    build_window_icon();
    // example:
    //   $env:MICROSWITCH_EMBED_CONFIG_PATH = 'example\config.yaml'
    //   cargo build -vv
    build_embedded_config();
}

#[cfg(not(target_os = "windows"))]
fn main() {
    build_window_icon();
    build_embedded_config();
}
