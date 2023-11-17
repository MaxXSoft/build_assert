fn main() {
  if let Ok(profile) = std::env::var("PROFILE") {
    println!("cargo:rustc-cfg=build={:?}", profile);
  }
}
