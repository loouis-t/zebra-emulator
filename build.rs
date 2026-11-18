// zpl-rs links libzpl dynamically but its rpaths only apply to its own targets,
// so the application must tell the loader where to find the library: next to the
// executable (Docker, distribution) and in zpl-rs's build output (local cargo builds).
fn main() {
    let origin = match std::env::var("CARGO_CFG_TARGET_OS").unwrap().as_str() {
        "macos" => "@executable_path",
        "linux" => "$ORIGIN",
        _ => return, // Windows: the DLL must sit next to the .exe or be on PATH.
    };
    let lib_dir = std::env::var("DEP_ZPL_LIB_DIR").unwrap();
    println!("cargo:rustc-link-arg=-Wl,-rpath,{origin}");
    println!("cargo:rustc-link-arg=-Wl,-rpath,{lib_dir}");
}
