fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    lute_src_rs::build_lute(lute_src_rs::LConfig::default());
} // ssssasshdsdsssfhhsss