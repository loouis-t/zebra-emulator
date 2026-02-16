use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

fn build_libzpl(target_dir: &Path) {
    let repo_dir = target_dir.join("go-zpl");
    let lib_output = target_dir.join("libzpl.a");

    // Clone repo if not exists
    if !repo_dir.exists() {
        println!("cargo:warning=Cloning go-zpl...");
        let status = Command::new("git")
            .args([
                "clone",
                "--depth",
                "1",
                "https://github.com/StirlingMarketingGroup/go-zpl",
            ])
            .arg(&repo_dir)
            .status()
            .expect("Failed to clone go-zpl");

        assert!(status.success());
    }

    if !lib_output.exists() {
        println!("cargo:warning=Building libzpl.a via Go...");

        let status = Command::new("go")
            .current_dir(repo_dir.join("cmd/libzpl"))
            .args([
                "build",
                "-buildmode=c-archive",
                "-o",
                lib_output.to_str().unwrap(),
            ])
            .status()
            .expect("Failed to run go build");

        assert!(status.success());
    }
}

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let target_dir = out_dir.ancestors().nth(3).unwrap();

    std::fs::create_dir_all(&target_dir).unwrap();

    // we rebuild libzpl ourselves to static link it
    build_libzpl(&target_dir);

    // Link static library
    println!("cargo:rustc-link-search=native={}", target_dir.display());
    println!("cargo:rustc-link-lib=static=zpl");

    // Go runtime dependencies
    println!("cargo:rustc-link-lib=dylib=pthread");
    println!("cargo:rustc-link-lib=dylib=dl");
    println!("cargo:rustc-link-lib=dylib=m");

    println!("cargo:rerun-if-changed=build.rs");
}
