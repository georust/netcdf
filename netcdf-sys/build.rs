fn main() {
    println!("cargo:rustc-link-lib=netcdf");
    println!("cargo:rerun-if-changed=build.rs");
}
