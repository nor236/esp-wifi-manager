fn main() {
    // esp-hal specific
    println!("cargo:rustc-link-arg=-Tlinkall.x");
    // println!("cargo:rustc-link-arg=-Tdefmt.x");

    println!("")
}
