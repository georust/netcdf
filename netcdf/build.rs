fn main() {
    if std::env::var("DEP_NETCDF_HAS_MMAP").is_ok() {
        println!("cargo:rustc-cfg=feature=\"has-mmap\"");
    }
    for (env, _value) in std::env::vars() {
        if let Some(version) = env.strip_prefix("DEP_NETCDF_VERSION_") {
            println!("cargo:rustc-cfg=feature={version}");
        }
    }
}
