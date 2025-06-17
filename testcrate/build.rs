fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    lute_src_rs::build_lute(lute_src_rs::LConfig {
        disable_crypto: true,
        ..Default::default()
    });
} // ssssasshdsdsssfhh
