use lute_src_rs_common::{cmake, cmake::Config, LConfig, commonflags::{build_cc_lute_lib, setup_lute_cmake}};
use std::env::current_dir;
use std::io::{Read, Write};

// Install (Linux)
// - g++-aarch64-linux-gnu
pub fn main() {
    // Get first arg if present
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 && args[1] == "upload" {
        // Upload prebuilts to git
        #[cfg(target_os = "linux")]
        {
            let targets = vec!["aarch64-unknown-linux-gnu", "x86_64-unknown-linux-gnu"];
            upload_to_git(targets, "linux");
        }

        #[cfg(target_os = "macos")]
        {
            let targets = vec!["aarch64-apple-macos"];
            upload_to_git(targets, "macos");
        }

        #[cfg(target_os = "windows")]
        {
            let targets = vec!["x86_64-pc-windows-msvc"];
            upload_to_git(targets, "windows");
        }

        return;
    }

    // On linux, build linux prebuilts for aarch64 and x86_64
    #[cfg(target_os = "linux")]
    {
        for target in vec!["aarch64-unknown-linux-gnu", "x86_64-unknown-linux-gnu"] {
            println!("Target: {}", target);
            build_lute_prebuilt(LConfig::default(), target, "linux");
        }
    }

    // On macos, build macos prebuilts for aarch64
    #[cfg(target_os = "macos")]
    {
        build_lute_prebuilt(LConfig::default(), "aarch64-apple-macos", "macos");
    }

    #[cfg(target_os = "windows")]
    {
        // On windows, build windows prebuilts for x86_64
        build_lute_prebuilt(LConfig::default(), "x86_64-pc-windows-msvc", "windows");
    }
}

pub fn upload_to_git(targets: Vec<&str>, os: &str) {
    // Remove the prebuilt-git directory if it exists
    let prebuilt_git_dir = "prebuilts-git";
    if std::path::Path::new(&prebuilt_git_dir).exists() {
        std::fs::remove_dir_all(&prebuilt_git_dir).expect("Failed to remove prebuilts-git directory");
    }

    // Git clone https://github.com/mluau/lute-prebuilts-{os}.git
    let git_url = format!("https://github.com/mluau/lute-prebuilts-{}.git", os);
    let output = std::process::Command::new("git")
        .arg("clone")
        .arg(&git_url)
        .arg(&prebuilt_git_dir)
        .output()
        .expect("Failed to clone lute-prebuilts repository");

    if !output.status.success() {
        panic!(
            "Failed to clone lute-prebuilts repository with stderr: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    // Copy the prebuilts/{target}/staticlibs directory to prebuilts-git/{target}
    for target in targets {
        let prebuilts_src_dir = format!("prebuilts/{}/build/staticlibs", target);
        let prebuilts_dest_dir = format!("{}/{}", prebuilt_git_dir, prebuilts_src_dir);
        println!("Copying from {} to {}", prebuilts_src_dir, prebuilts_dest_dir);
        // Create the destination directory if it doesn't exist
        std::fs::create_dir_all(&prebuilts_dest_dir).expect("Failed to create destination directory");

        // Copy the contents of the source directory to the destination directory
        for entry in std::fs::read_dir(&prebuilts_src_dir).expect("Failed to read source directory") {
            let entry = entry.expect("Failed to read entry");
            let src_path = entry.path();
            let dest_path = std::path::Path::new(&prebuilts_dest_dir).join(entry.file_name());

            println!("Copying {} to {}", src_path.display(), dest_path.display());

            // Split the file into parts if it is larger than 100MB
            if src_path.is_file() && src_path.metadata().unwrap().len() > 100  * 1024 * 1024 {
                println!("Splitting file {} into  parts", src_path.display());
                let file = std::fs::File::open(&src_path).expect("Failed to open file");
                let mut reader = std::io::BufReader::new(file);
                let mut buffer = vec![0; 90 * 1024 * 1024]; // 100MB buffer
                let mut part_number = 1;
                loop {
                    let bytes_read = reader.read(&mut buffer).expect("Failed to read file");
                    if bytes_read == 0 {
                        break; // End of file
                    }
                    let part_file_name = format!("{}.part{}", dest_path.display(), part_number);
                    let mut part_file = std::fs::File::create(&part_file_name).expect("Failed to create part file");
                    part_file.write_all(&buffer[..bytes_read]).expect("Failed to write to part file");
                    println!("Created part file: {}", part_file_name);
                    part_number += 1;
                }
            } else {
                std::fs::copy(&src_path, &dest_path).expect("Failed to copy file");
            }
        }
    }

    println!("Uploading prebuilts to {}...", prebuilt_git_dir);

    // Add, commit and push the changes to the prebuilts-git repository
    let output = std::process::Command::new("git")
        .arg("-C")
        .arg(&prebuilt_git_dir)
        .arg("add")
        .arg(".")
        .output()
        .expect("Failed to add changes to git");

    if !output.status.success() {
        panic!(
            "Failed to add changes to git with stderr: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let output = std::process::Command::new("git")
        .arg("-C")
        .arg(&prebuilt_git_dir)
        .arg("commit")
        .arg("-m")
        .arg("Update prebuilts")
        .output()
        .expect("Failed to commit changes to git");

    if !output.status.success() {
        panic!(
            "Failed to commit changes to git with stderr: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let output = std::process::Command::new("git")
        .arg("-C")
        .arg(&prebuilt_git_dir)
        .arg("push")
        .arg("origin")
        .arg("main") // Assuming the main branch is called "main"
        .output()
        .expect("Failed to push changes to git");
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

pub fn build_lute_prebuilt(lcfg: LConfig, target: &str, os: &str) {
    let host = env!("HOST_VAR");
    println!("Host: {}", host);

    // Make prebuilts/{target} directory if it doesn't exist
    let prebuilts_dir = format!("prebuilts/{}/build", target);
    std::fs::create_dir_all(&prebuilts_dir).expect("Failed to create prebuilts directory");

    unsafe {
        std::env::set_var("HOST", host);
        std::env::set_var("TARGET", target);
        std::env::set_var("OPT_LEVEL", "3");
        std::env::set_var("OUT_DIR", &prebuilts_dir);

        if os == "windows" {
            std::env::set_var("CARGO_CFG_TARGET_ENV", "msvc");
            std::env::set_var("CARGO_CFG_TARGET_OS", "windows");
            std::env::set_var("CARGO_CFG_TARGET_ARCH", "x86_64");
        } else if os == "macos" {
            std::env::set_var("CARGO_CFG_TARGET_ENV", "darwin");
            std::env::set_var("CARGO_CFG_TARGET_OS", "macos");
            std::env::set_var("CARGO_CFG_TARGET_ARCH", "aarch64");
        } else {
            std::env::set_var("CARGO_CFG_TARGET_ENV", target.split('-').nth(1).unwrap_or("unknown"));
            std::env::set_var("CARGO_CFG_TARGET_OS", target.split('-').nth(2).unwrap_or("unknown"));
            std::env::set_var("CARGO_CFG_TARGET_ARCH", target.split('-').nth(0).unwrap_or("unknown"));
        }
    }

    // Switch directory to CARGO_MANIFEST_DIR
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
        .std("c++20")
        .target(&target)
        .cpp(true);

    if target.ends_with("emscripten") {
        // Enable c++ exceptions for emscripten (it's disabled by default)
        // Later we should switch to wasm exceptions
        config.flag_if_supported("-fexceptions");
    }

    // Custom is a special library that needs to be built manually and linked in as well
    println!("Building Luau.Custom for target: {}", target);
    
    build_cc_lute_lib(
        lcfg,
        "Luau.Custom",
        vec!["Custom/src/lextra.cpp".to_string(), "Custom/src/lflags.cpp".to_string()]
    );
    
    // Also build LuteExt
    println!("Building Luau.LuteExt for target: {}", target);

    build_cc_lute_lib(
        lcfg,
        "Luau.LuteExt",
        vec!["LuteExt/src/lopen.cpp".to_string()]
    );

    let dst = setup_lute_cmake(lcfg);

    // Now copy the final output files to the prebuilts directory/{target}/staticlibs
    //
    // On linux, these will be *.a files
    // On macos, these will be *.a files
    // On windows, these will be *.lib files
    let staticlibs_dir = format!("{}/staticlibs", prebuilts_dir);
    std::fs::create_dir_all(&staticlibs_dir).expect("Failed to create staticlibs directory");

    // Now glob
    let ending = if os == "windows" {
        "lib"
    } else {
        "a"
    };

    // Copy all static libraries from the build directory to the staticlibs directory
    let files = glob::glob(&format!("{}/**/*.{}",  prebuilts_dir, ending))
    .expect("Failed to glob for static libraries");

    std::thread::sleep(std::time::Duration::from_millis(100)); // Sleep to avoid windows issues

    files
        .filter_map(Result::ok)
        .for_each(|path| {
            if path.display().to_string().starts_with(&staticlibs_dir) || path.display().to_string().contains("staticlibs") {
                // Skip files that are already in the staticlibs directory
                return;
            }
            let file_name = path.file_name().unwrap();
            let dest_path = std::path::Path::new(&staticlibs_dir).join(file_name);
            println!("Copying {} to {}", path.display(), dest_path.display());
            std::fs::copy(path.clone(), &dest_path).expect("Failed to copy static library");
        });
}
