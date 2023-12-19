fn main() {
    if std::env::var("DEP_NETCDF_HAS_MMAP").is_ok() {
        println!("cargo:rustc-cfg=feature=\"has-mmap\"");
    }
}
