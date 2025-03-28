fn main() {
    println!("cargo:rustc-link-lib=framework=SkyLight");
    println!("cargo:rustc-link-search=framework=/System/Library/PrivateFrameworks");
}
