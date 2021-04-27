extern crate image;
use std::{env, fs};
use std::path::PathBuf;

fn package_dir() -> String {
    env::var("CARGO_MANIFEST_DIR").expect("No CARGO_MANIFEST_DIR env var")
}

fn build_window_icon() {
    let out_dir = env::var("OUT_DIR").expect("No OUT_DIR env var");
    let img_path: PathBuf = [package_dir().as_str(), "resources", "microswitch-icon-32.png"].iter().collect();
    let out_path: PathBuf = [out_dir.as_str(), "microswitch-icon-32-rgba"].iter().collect();

    let img = image::open(img_path).expect("Failed to read/decode microswitch-icon-32-rgba");
    let img = img.to_rgba8();
    let rgba = img.into_raw();
    println!("DEBUG: writing to {}", out_path.to_str().unwrap());
    fs::write(&out_path, rgba).expect("Failed to write to microswitch-icon-32-rgba");
}

#[cfg(target_os = "windows")]
fn main() {
    let package_dir = package_dir();
    let resources_dir: PathBuf = [package_dir.as_str(), "resources"].iter().collect();

    println!("cargo:rustc-link-search=native={}", resources_dir.to_str().unwrap());
    println!("cargo:rustc-link-lib=dylib={}", "resources");

    build_window_icon();
}

#[cfg(not(target_os = "windows"))]
fn main() {
    build_window_icon();
}
