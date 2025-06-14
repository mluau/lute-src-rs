use cmake::Config;

pub fn build_lute() {
    println!("cargo:rerun-if-changed=build.rs");
    //println!("cargo:rerun-if-changed=build_hash.txt");

    println!("cargo:rustc-env=LUAU_VERSION=0.677"); // TODO: Update when needed

    // Switch directory to CARGO_MANIFEST_DIR
    std::env::set_current_dir(env!("CARGO_MANIFEST_DIR")).unwrap();
    // This is needed to run the luthier.py script
    println!("Current directory: {}", std::env::current_dir().unwrap().display());

    // Check that python is installed, error if not. This is needed
    // for luthier.py to fetch dependencies
    if std::process::Command::new("python3")
        .arg("--version")
        .output()
        .is_err()
    {
        panic!("Python 3 is required to build the lute runtime");
    }

    // Use tools/luthier.py in the lute folder to fetch dependencies
    let output = std::process::Command::new("lute")
        .current_dir("lute")
        .arg("tools/luthier.luau")
        .arg("fetch")
        .arg("lute")
        .output()
        .expect("Failed to run tools/luthier.py fetch lute");

    if !output.status.success() {
        panic!("Failed to run tools/luthier.py fetch lute with stderr: {}", String::from_utf8_lossy(&output.stderr));
    }

    let output = std::process::Command::new("lute")
        .current_dir("lute")
        .arg("tools/luthier.luau")
        .arg("generate")
        .arg("lute")
        .output()
        .expect("Failed to run tools/luthier.py fetch lute");

    if !output.status.success() {
        panic!("Failed to run tools/luthier.py generate lute with stderr: {}", String::from_utf8_lossy(&output.stderr));
    }

    //panic!("Cannot build lute runtime yet, please run `cargo build` manually");
    
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
        .define("LUAU_EXTERN_C", "ON") // Provides DLUA_USE_LONGJMP, DLUA_API, LUACODE_API, LUACODEGEN_API
        .define("LUAU_STATIC_CRT", "ON")
        .define("LUAU_BUILD_STATIC", "ON")
        .define("WITH_ZLIB", "OFF")
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
        .flag("-g")
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

    cc::Build::new() 
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
        .flag("-DLUA_LUTAG_LIMIT=256") // 128 is default, but we want 256 to give 128 for mlua and 128 to lute
        .flag("-fexceptions")
        .flag("-g")
        .compile("Luau.LuteExt");

    println!("cargo:rustc-link-lib=dylib=stdc++");
    println!("cargo:rustc-link-search=native={}/build", dst.display());
    println!("cargo:rustc-link-search=native={}/build/extern/luau", dst.display());
    println!("cargo:rustc-link-search=native={}/build/lute/crypto", dst.display());
    println!("cargo:rustc-link-search=native={}/build/lute/fs", dst.display());
    println!("cargo:rustc-link-search=native={}/build/lute/luau", dst.display());
    println!("cargo:rustc-link-search=native={}/build/lute/net", dst.display());
    println!("cargo:rustc-link-search=native={}/build/lute/process", dst.display());
    println!("cargo:rustc-link-search=native={}/build/lute/runtime", dst.display());
    println!("cargo:rustc-link-search=native={}/build/lute/require", dst.display());
    println!("cargo:rustc-link-search=native={}/build/lute/std", dst.display());
    println!("cargo:rustc-link-search=native={}/build/lute/system", dst.display());
    println!("cargo:rustc-link-search=native={}/build/lute/task", dst.display());
    println!("cargo:rustc-link-search=native={}/build/lute/time", dst.display());
    println!("cargo:rustc-link-search=native={}/build/lute/vm", dst.display());

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
    println!("cargo:rustc-link-lib=static=Lute.Crypto");
    println!("cargo:rustc-link-lib=static=Lute.Fs");
    println!("cargo:rustc-link-lib=static=Lute.Luau");
    println!("cargo:rustc-link-lib=static=Lute.Net");
    println!("cargo:rustc-link-lib=static=Lute.Process");
    println!("cargo:rustc-link-lib=static=Lute.Runtime");
    println!("cargo:rustc-link-lib=static=Lute.Require");
    println!("cargo:rustc-link-lib=static=Lute.Std");
    println!("cargo:rustc-link-lib=static=Lute.System");
    println!("cargo:rustc-link-lib=static=Lute.Task");
    println!("cargo:rustc-link-lib=static=Lute.Time");
    println!("cargo:rustc-link-lib=static=Lute.VM");
    println!("cargo:rustc-link-lib=static=uSockets");

    // boringssl
    println!("cargo:rustc-link-search=native={}/build/extern/boringssl", dst.display());
    println!("cargo:rustc-link-lib=static=crypto");
    println!("cargo:rustc-link-lib=static=decrepit");
    println!("cargo:rustc-link-lib=static=pki");
    println!("cargo:rustc-link-lib=static=ssl");
println!("cargo:rustc-link-lib=static=sodium");

    // curl
    println!("cargo:rustc-link-search=native={}/build/extern/curl/lib", dst.display());
    
    // Debug
    let binding = Config::new("lute");
    let profile = binding.get_profile();
    if profile == "Debug" {
        println!("cargo:rustc-link-lib=static=curl-d");
    } else {
        println!("cargo:rustc-link-lib=static=curl");
    }

    // libuv
    println!("cargo:rustc-link-search=native={}/build/extern/libuv", dst.display());
    println!("cargo:rustc-link-lib=static=uv");

    // zlib (system)
    println!("cargo:rustc-link-search=native={}/build/extern/zlib", dst.display());
    println!("cargo:rustc-link-lib=static=z");
}

