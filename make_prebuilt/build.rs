// Only used to propogate HOST and other vars
pub fn main() {
    let host = std::env::var("HOST").unwrap_or_else(|_| "unknown".to_string());
    // Propogate PREBUILT_HOST environment variable 
    println!("cargo:rustc-env=HOST_VAR={}", host);
}