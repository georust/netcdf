use semver::Version;

fn main() {
    println!("cargo::rustc-check-cfg=cfg(feature, values(\"has-mmap\"))");
    let versions = [
        Version::new(4, 4, 0),
        Version::new(4, 4, 1),
        Version::new(4, 5, 0),
        Version::new(4, 6, 0),
        Version::new(4, 6, 1),
        Version::new(4, 6, 2),
        Version::new(4, 6, 3),
        Version::new(4, 7, 0),
        Version::new(4, 7, 1),
        Version::new(4, 7, 2),
        Version::new(4, 7, 3),
        Version::new(4, 7, 4),
        Version::new(4, 8, 0),
        Version::new(4, 8, 1),
        Version::new(4, 9, 0),
        Version::new(4, 9, 1),
        Version::new(4, 9, 2),
        // Keep this list up to date with netcdf-sys/build.rs
    ];

    for version in &versions {
        println!("cargo::rustc-check-cfg=cfg(feature, values(\"{version}\"))");
    }

    if std::env::var("DEP_NETCDF_HAS_MMAP").is_ok() {
        println!("cargo::rustc-cfg=feature=\"has-mmap\"");
    }
    for (env, _value) in std::env::vars() {
        if let Some(version) = env.strip_prefix("DEP_NETCDF_VERSION_") {
            println!("cargo:rustc-cfg=feature={version}");
        }
    }
}
