use cmake::Config;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    //println!("cargo:rerun-if-changed=build_hash.txt");

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
    let output = std::process::Command::new("python3")
        .current_dir("lute")
        .arg("tools/luthier.py")
        .arg("fetch")
        .arg("lute")
        .output()
        .expect("Failed to run tools/luthier.py fetch lute");

    if !output.status.success() {
        panic!("Failed to run tools/luthier.py fetch lute with stderr: {}", String::from_utf8_lossy(&output.stderr));
    }

    let output = std::process::Command::new("python3")
        .current_dir("lute")
        .arg("tools/luthier.py")
        .arg("generate")
        .arg("lute")
        .output()
        .expect("Failed to run tools/luthier.py fetch lute");

    if !output.status.success() {
        panic!("Failed to run tools/luthier.py generate lute with stderr: {}", String::from_utf8_lossy(&output.stderr));
    }

    //panic!("Cannot build lute runtime yet, please run `cargo build` manually");
    
    let dst = Config::new("lute")
        .define("LUAU_EXTERN_C", "ON")
        .define("LUAU_STATIC_CRT", "ON")
        .no_build_target(true)
        .build();

    println!("cargo:rustc-link-search=native={}/build", dst.display());
    println!("cargo:rustc-link-search=native={}/build/extern/luau", dst.display());
    
    println!("cargo:rustc-link-lib=static=Luau.Analysis");
    println!("cargo:rustc-link-lib=static=Luau.Ast");
    println!("cargo:rustc-link-lib=static=Luau.CodeGen");
    println!("cargo:rustc-link-lib=static=Luau.Config");
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

    // curl
    println!("cargo:rustc-link-search=native={}/build/extern/curl/lib", dst.display());
    println!("cargo:rustc-link-lib=static=curl-d");

    // libuv
    println!("cargo:rustc-link-search=native={}/build/extern/libuv", dst.display());
    println!("cargo:rustc-link-lib=static=uv");
}

