use lute_src_rs_common::{
    cmake::Config,
    finalize::finalize_build,
    commonflags::{build_cc_lute_lib, setup_lute_cmake}
};
pub use lute_src_rs_common::LConfig;
use std::env::current_dir;

fn does_lute_exist(cmd: &str) -> bool {
    let Ok(cmd) = std::process::Command::new(cmd)
    .arg("run")
    .arg("test.luau")
    .status() else {
        return false; // Command not found
    };

    cmd.success() // If the command exists, it should return success
}

pub fn build_lute(lcfg: LConfig) {
    println!("cargo:rustc-env=LUAU_VERSION=0.677"); // TODO: Update when needed

    // Switch directory to CARGO_MANIFEST_DIR
    std::env::set_current_dir(env!("CARGO_MANIFEST_DIR")).unwrap();
    // This is needed to run the luthier.py script
    println!(
        "Current directory: {}",
        std::env::current_dir().unwrap().display()
    );

    // Check for lute/.done_luthier file to see if we need to run luthier.luau (which is slow)
    let lute_done_path = std::path::Path::new("lute/.done_luthier");
    if !lute_done_path.exists() {
    // Check that python is installed, error if not. This is needed
    // for luthier.py to fetch dependencies
    let lute = if does_lute_exist("lute") {
            "lute".to_string()
        } else if does_lute_exist("lute.exe") {
            "lute.exe".to_string()
        } else if cfg!(all(target_os = "windows", target_arch = "x86_64")) {
            format!("{}/lute-bins/lute-windows-x86_64.exe", current_dir().unwrap().display()) // prebuilt lute binary for Windows x86_64
        } else if cfg!(all(target_os = "linux", target_arch = "x86_64")) {
            format!("{}/lute-bins/lute-linux-x86_64", current_dir().unwrap().display()) // prebuilt lute binary for Linux x86_64
        } else if cfg!(all(target_os = "linux", target_arch = "aarch64")) {
            format!("{}/lute-bins/lute-linux-aarch64", current_dir().unwrap().display()) // prebuilt lute binary for Linux aarch64
        } else if cfg!(all(target_os = "macos", target_arch = "aarch64")) {
            format!("{}/lute-bins/lute-macos-aarch64", current_dir().unwrap().display()) // prebuilt lute binary for macOS aarch64
        } else {
            panic!("Lute binary not found and pre-built binaries are not available for this platform. Please build Lute manually and add it to your path as it is required for bootstrapping itself.");
        };

        println!("Using lute binary: {}", lute);

        // Use tools/luthier.py in the lute folder to fetch dependencies
        let output = std::process::Command::new(&lute)
            .current_dir("lute")
            .arg("tools/luthier.luau")
            .arg("fetch")
            .arg("lute")
            .output()
            .expect("Failed to run tools/luthier.py fetch lute");

        if !output.status.success() {
            panic!(
                "Failed to run tools/luthier.py fetch lute with stderr: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }

        let output = std::process::Command::new(lute)
            .current_dir("lute")
            .arg("tools/luthier.luau")
            .arg("generate")
            .arg("lute")
            .output()
            .expect("Failed to run tools/luthier.py fetch lute");

        if !output.status.success() {
            panic!(
                "Failed to run tools/luthier.py generate lute with stderr: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }

        // Create the .done_luthier file to indicate that luthier.py has been run
        std::fs::File::create(lute_done_path).expect("Failed to create .done_luthier file");
    }

    // Configure C++
    let dst = setup_lute_cmake(lcfg);

    // Custom is a special library that needs to be built manually and linked in as well
    build_cc_lute_lib(
        lcfg,
        "Luau.Custom",
        vec!["Custom/src/lextra.cpp".to_string(), "Custom/src/lflags.cpp".to_string()]
    );
    
    // Also build LuteExt
    build_cc_lute_lib(
        lcfg,
        "Luau.LuteExt",
        vec!["LuteExt/src/lopen.cpp".to_string()]
    );

    println!("cargo:rustc-link-search=native={}/build", dst.display());
    
    #[cfg(not(target_os = "windows"))]
    {
        println!(
            "cargo:rustc-link-search=native={}/build/extern/luau",
            dst.display()
        );

        if !lcfg.disable_crypto {
            println!(
                "cargo:rustc-link-search=native={}/build/lute/crypto",
                dst.display()
            );
        }
        println!(
            "cargo:rustc-link-search=native={}/build/lute/fs",
            dst.display()
        );
        println!(
            "cargo:rustc-link-search=native={}/build/lute/luau",
            dst.display()
        );
        if !lcfg.disable_net {
            println!(
                "cargo:rustc-link-search=native={}/build/lute/net",
                dst.display()
            );
        }
        println!(
            "cargo:rustc-link-search=native={}/build/lute/process",
            dst.display()
        );
        println!(
            "cargo:rustc-link-search=native={}/build/lute/runtime",
            dst.display()
        );
        println!(
            "cargo:rustc-link-search=native={}/build/lute/require",
            dst.display()
        );
        println!(
            "cargo:rustc-link-search=native={}/build/lute/std",
            dst.display()
        );
        println!(
            "cargo:rustc-link-search=native={}/build/lute/system",
            dst.display()
        );
        println!(
            "cargo:rustc-link-search=native={}/build/lute/task",
            dst.display()
        );
        println!(
            "cargo:rustc-link-search=native={}/build/lute/time",
            dst.display()
        );
        println!(
            "cargo:rustc-link-search=native={}/build/lute/vm",
            dst.display()
        );
    }

    #[cfg(target_os = "windows")]
    {
        println!("cargo:rustc-link-search=native={}/build/Release", dst.display());
        println!("cargo:rustc-link-search=native={}/build/extern/luau/Release", dst.display());
        println!("cargo:rustc-link-search=native={}/build/extern/libuv/Release", dst.display());

        if !lcfg.disable_crypto {
            println!(
                "cargo:rustc-link-search=native={}/build/lute/crypto/Release",
                dst.display()
            );
        }
        println!(
            "cargo:rustc-link-search=native={}/build/lute/fs/Release",
            dst.display()
        );
        println!(
            "cargo:rustc-link-search=native={}/build/lute/luau/Release",
            dst.display()
        );
        if !lcfg.disable_net {
            println!(
                "cargo:rustc-link-search=native={}/build/lute/net/Release",
                dst.display()
            );
        }
        println!(
            "cargo:rustc-link-search=native={}/build/lute/process/Release",
            dst.display()
        );
        println!(
            "cargo:rustc-link-search=native={}/build/lute/runtime/Release",
            dst.display()
        );
        println!(
            "cargo:rustc-link-search=native={}/build/lute/require/Release",
            dst.display()
        );
        println!(
            "cargo:rustc-link-search=native={}/build/lute/std/Release",
            dst.display()
        );
        println!(
            "cargo:rustc-link-search=native={}/build/lute/system/Release",
            dst.display()
        );
        println!(
            "cargo:rustc-link-search=native={}/build/lute/task/Release",
            dst.display()
        );
        println!(
            "cargo:rustc-link-search=native={}/build/lute/time/Release",
            dst.display()
        );
        println!(
            "cargo:rustc-link-search=native={}/build/lute/vm/Release",
            dst.display()
        );
    }

    if !lcfg.disable_net || !lcfg.disable_crypto {
        // boringssl
        #[cfg(not(target_os = "windows"))]
        println!(
            "cargo:rustc-link-search=native={}/build/extern/boringssl",
            dst.display()
        );

        #[cfg(target_os = "windows")]
        println!(
            "cargo:rustc-link-search=native={}/build/extern/boringssl/Release",
            dst.display()
        );
    }

    if !lcfg.disable_net {
        // curl
        println!(
            "cargo:rustc-link-search=native={}/build/extern/curl/lib",
            dst.display()
        );
    }
    
    // libuv
    #[cfg(not(target_os = "windows"))]
    {
        println!(
            "cargo:rustc-link-search=native={}/build/extern/libuv",
            dst.display()
        );
    }

    // zlib (system)
    if !lcfg.disable_net {
        println!(
            "cargo:rustc-link-search=native={}/build/extern/zlib",
            dst.display()
        );
    }

    finalize_build(lcfg, false);
}