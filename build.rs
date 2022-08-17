fn main() {
    println!("cargo:rustc-link-lib=dylib=minizip");
    println!("cargo:rustc-link-lib=static=z");
}
