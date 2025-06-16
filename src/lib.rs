use cmake::Config;

use rustc_version::{version_meta, Channel};
use std::env::current_dir;

#[derive(Clone, Copy)]
pub struct LConfig {
    pub disable_crypto: bool,
    pub disable_net: bool
}

impl Default for LConfig {
    fn default() -> Self {
        Self {
            disable_crypto: true, // Takes too long to build
            disable_net: true, // Takes too long to build
        }
    }
}

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

    // On non-nightly builds, we need to use the lld linker
    match version_meta().unwrap().channel {
        Channel::Nightly | Channel::Dev => {}
        _ => {
            println!("cargo:rustc-link-arg=-fuse-ld=lld");
        }
    }

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
    let mut config = cc::Build::new();
    config
        .warnings(false)
        .cargo_metadata(true)
        .std("c++17")
        .cpp(true);

    let target = std::env::var("TARGET").unwrap();

    if target.ends_with("emscripten") {
        // Enable c++ exceptions for emscripten (it's disabled by default)
        // Later we should switch to wasm exceptions
        config.flag_if_supported("-fexceptions");
    }

    let dst = Config::new("lute")
        .profile("Release") // Debug builds tend to be extremely slow and nearly unusable in practice
        .define("LUAU_EXTERN_C", "ON") // Provides DLUA_USE_LONGJMP, DLUA_API, LUACODE_API, LUACODEGEN_API
        .define("LUAU_STATIC_CRT", "ON")
        .define("LUAU_BUILD_STATIC", "ON")
        .define("LUTE_DISABLE_NET", if lcfg.disable_net { "ON" } else { "OFF" } )
        .define("LUTE_DISABLE_CRYPTO", if lcfg.disable_crypto { "ON" } else { "OFF" }  )
        .cxxflag("-DLUAI_MAXCSTACK=1000000")
        .cxxflag("-DLUA_UTAG_LIMIT=255") // 128 is default, but we want 255 to give 128 for mlua and 128 to lute
        .cxxflag("-DLUA_LUTAG_LIMIT=255") // 128 is default, but we want 255 to give 128 for mlua and 128 to lute
        .init_cxx_cfg(config)
        .no_build_target(true)
        .build();

    // Custom is a special library that needs to be built manually and linked in as well
    cc::Build::new()
        .cpp(true)
        .file("Custom/src/lextra.cpp")
        .file("Custom/src/lflags.cpp")
        .flag("-DLUA_USE_LONGJMP=1")
        .flag("-DLUA_API=extern \"C\"")
        .flag("-DLUACODE_API=extern \"C\"")
        .flag("-DLUACODEGEN_API=extern \"C\"")
        .flag("-DLUAI_MAXCSTACK=1000000")
        .flag("-DLUA_UTAG_LIMIT=256") // 128 is default, but we want 256 to give 128 for mlua and 128 to lute
        .flag("-DLUA_LUTAG_LIMIT=256") // 128 is default, but we want 256 to give 128 for mlua and 128 to lute
        .flag("-fexceptions")
        .include("lute/extern/luau/VM/include")
        .include("lute/extern/luau/VM/src")
        .include("lute/extern/luau/Common/include")
        .include("lute/extern/luau/Compiler/include")
        .compile("Luau.Custom");

    // Also build LuteExt

    /*
    target_compile_definitions(Luau.VM PUBLIC LUA_USE_LONGJMP=1)
    target_compile_definitions(Luau.VM PUBLIC LUA_API=extern\"C\")
    target_compile_definitions(Luau.Compiler PUBLIC LUACODE_API=extern\"C\")
    target_compile_definitions(Luau.CodeGen PUBLIC LUACODEGEN_API=extern\"C\")
    */

    let mut build = cc::Build::new();

    build
        .cpp(true)
        .file("LuteExt/src/lopen.cpp")
        .include("lute/lute/cli/include")
        .include("lute/lute/crypto/include")
        .include("lute/lute/fs/include")
        .include("lute/lute/luau/include")
        .include("lute/lute/net/include")
        .include("lute/lute/process/include")
        .include("lute/lute/system/include")
        .include("lute/lute/vm/include")
        .include("lute/lute/task/include")
        .include("lute/lute/time/include")
        .include("lute/lute/runtime/include")
        .include("lute/extern/luau/VM/include")
        .include("lute/extern/luau/VM/src")
        .include("lute/extern/luau/Common/include")
        .include("lute/extern/luau/Compiler/include")
        .include("lute/extern/libuv/include")
        .flag("-DLUA_USE_LONGJMP=1")
        .flag("-DLUA_API=extern \"C\"")
        .flag("-DLUACODE_API=extern \"C\"")
        .flag("-DLUACODEGEN_API=extern \"C\"")
        .flag("-DLUAI_MAXCSTACK=1000000")
        .flag("-DLUA_UTAG_LIMIT=256") // 128 is default, but we want 256 to give 128 for mlua and 128 to lute
        .flag("-DLUA_LUTAG_LIMIT=256"); // 128 is default, but we want 256 to give 128 for mlua and 128 to lute
        
    if lcfg.disable_net {
        build.flag("-DLUTE_DISABLE_NET=1");
    }

    if lcfg.disable_crypto {
        build.flag("-DLUTE_DISABLE_CRYPTO=1");
    }

    build
        .flag("-fexceptions")
        .compile("Luau.LuteExt");

    println!("cargo:rustc-link-search=native={}/build", dst.display());
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

    println!("cargo:rustc-link-lib=static=Luau.Ast");
    println!("cargo:rustc-link-lib=static=Luau.Analysis");
    println!("cargo:rustc-link-lib=static=Luau.CodeGen");
    println!("cargo:rustc-link-lib=static=Luau.Config");
    println!("cargo:rustc-link-lib=static=Luau.Compiler");
    println!("cargo:rustc-link-lib=static=Luau.CLI.lib");
    println!("cargo:rustc-link-lib=static=Luau.EqSat");
    println!("cargo:rustc-link-lib=static=Luau.Require");
    println!("cargo:rustc-link-lib=static=Luau.RequireNavigator");
    println!("cargo:rustc-link-lib=static=Luau.VM");
    if !lcfg.disable_crypto {
        println!("cargo:rustc-link-lib=static=Lute.Crypto");
    }
    println!("cargo:rustc-link-lib=static=Lute.Fs");
    println!("cargo:rustc-link-lib=static=Lute.Luau");
    if !lcfg.disable_net {
        println!("cargo:rustc-link-lib=static=Lute.Net");
    }
    println!("cargo:rustc-link-lib=static=Lute.Process");
    println!("cargo:rustc-link-lib=static=Lute.Runtime");
    println!("cargo:rustc-link-lib=static=Lute.Require");
    println!("cargo:rustc-link-lib=static=Lute.Std");
    println!("cargo:rustc-link-lib=static=Lute.System");
    println!("cargo:rustc-link-lib=static=Lute.Task");
    println!("cargo:rustc-link-lib=static=Lute.Time");
    println!("cargo:rustc-link-lib=static=Lute.VM");
    
    if !lcfg.disable_net {
        println!("cargo:rustc-link-lib=static=uSockets");
    }

    if !lcfg.disable_net || !lcfg.disable_crypto {
        // boringssl
        println!(
            "cargo:rustc-link-search=native={}/build/extern/boringssl",
            dst.display()
        );
        println!("cargo:rustc-link-lib=static=crypto");
        println!("cargo:rustc-link-lib=static=decrepit");
        println!("cargo:rustc-link-lib=static=pki");
        println!("cargo:rustc-link-lib=static=ssl");
    }

    if !lcfg.disable_crypto {
        // libsodium
        println!("cargo:rustc-link-lib=static=sodium");
    }

    if !lcfg.disable_net {
        // curl
        println!(
            "cargo:rustc-link-search=native={}/build/extern/curl/lib",
            dst.display()
        );
    }

    
    if !lcfg.disable_net {
        // Curl
        let binding = Config::new("lute");
        let profile = binding.get_profile();
        if profile == "Debug" {
            println!("cargo:rustc-link-lib=static=curl-d");
        } else {
            println!("cargo:rustc-link-lib=static=curl");
        }
    }

    // libuv
    println!(
        "cargo:rustc-link-search=native={}/build/extern/libuv",
        dst.display()
    );
    println!("cargo:rustc-link-lib=static=uv");

    // zlib (system)
    if !lcfg.disable_net {
        println!(
            "cargo:rustc-link-search=native={}/build/extern/zlib",
            dst.display()
        );
        println!("cargo:rustc-link-lib=static=z");
    }
}
