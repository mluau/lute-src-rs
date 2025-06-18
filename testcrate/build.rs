fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    
    #[cfg(not(feature = "prebuilt"))]
    lute_src_rs::build_lute(lute_src_rs::LConfig {
        disable_crypto: true,
        ..Default::default()
    });

    #[cfg(feature = "prebuilt")]
    lute_prebuilts_chooser::integrate();
} // ssssasshdsdsssfhhss