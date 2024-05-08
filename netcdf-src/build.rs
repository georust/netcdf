macro_rules! feature {
    ($feature:expr) => {
        std::env::var(concat!("CARGO_FEATURE_", $feature))
    };
}

fn get_hdf5_version() -> String {
    let (major, minor, patch) = std::env::vars()
        .filter_map(|(key, value)| {
            key.strip_prefix("DEP_HDF5_VERSION_").map(|key| {
                assert_eq!(value, "1");
                let mut version = key.split('_');
                let major: usize = version.next().unwrap().parse().unwrap();
                let minor: usize = version.next().unwrap().parse().unwrap();
                let patch: usize = version.next().unwrap().parse().unwrap();

                (major, minor, patch)
            })
        })
        .max()
        .expect("Crate hdf5 should have emitted a hdf5 version");

    format!("{major}.{minor}.{patch}")
}

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    let hdf5_incdir = std::env::var("DEP_HDF5_INCLUDE").unwrap();
    let mut hdf5_lib = std::env::var("DEP_HDF5_LIBRARY").unwrap();
    let mut hdf5_hl_lib = std::env::var("DEP_HDF5_HL_LIBRARY").unwrap();

    #[cfg(unix)]
    {
        let hdf5_root = format!("{hdf5_incdir}/../");
        let mut hdf5_libdir = format!("{hdf5_root}/lib/");
        if !std::path::Path::new(&hdf5_libdir).exists() {
            hdf5_libdir = format!("{hdf5_root}/lib64/");
        }
        hdf5_lib = format!("{hdf5_libdir}/{hdf5_lib}.a");
        hdf5_hl_lib = format!("{hdf5_libdir}/{hdf5_hl_lib}.a");
    }

    let hdf5_version = get_hdf5_version();

    let mut netcdf_config = cmake::Config::new("source");
    netcdf_config
        .define("BUILD_SHARED_LIBS", "OFF")
        .define("NC_FIND_SHARED_LIBS", "OFF")
        .define("BUILD_UTILITIES", "OFF")
        .define("ENABLE_EXAMPLES", "OFF")
        .define("ENABLE_DAP_REMOTE_TESTS", "OFF")
        .define("ENABLE_TESTS", "OFF")
        .define("ENABLE_EXTREME_NUMBERS", "OFF")
        .define("ENABLE_PARALLEL_TESTS", "OFF")
        .define("ENABLE_FILTER_TESTING", "OFF")
        .define("ENABLE_BASH_SCRIPT_TESTING", "OFF")
        .define("ENABLE_PLUGINS", "OFF")
        .define("PLUGIN_INSTALL_DIR", "OFF")
        //
        .define("HDF5_VERSION", &hdf5_version)
        .define("HDF5_C_LIBRARY", &hdf5_lib)
        .define("HDF5_HL_LIBRARY", &hdf5_hl_lib)
        .define("HDF5_INCLUDE_DIR", hdf5_incdir)
        //
        .define("ENABLE_LIBXML2", "OFF") // Use bundled xml2
        //
        .define("ENABLE_PARALLEL4", "OFF") // TODO: Enable mpi support
        //
        .define("ENABLE_NCZARR", "OFF") // TODO: requires a bunch of deps
        //
        .define("ENABLE_DAP", "OFF") // TODO: feature flag, requires curl
        .define("ENABLE_BYTERANGE", "OFF") // TODO: feature flag, requires curl
        .define("ENABLE_DAP_REMOTE_TESTS", "OFF")
        //
        .profile("RelWithDebInfo"); // TODO: detect opt-level

    let zlib_include_dir = std::env::var("DEP_Z_INCLUDE").unwrap();
    netcdf_config.define("ZLIB_ROOT", format!("{zlib_include_dir}/.."));

    if feature!("DAP").is_ok() {
        netcdf_config.define("ENABLE_DAP", "ON");
        netcdf_config.define("ENABLE_BYTERANGE", "ON");
    }

    let netcdf = netcdf_config.build();

    println!("cargo:lib=netcdf");
    let search_path = format!("{}/lib", netcdf.display());
    if std::path::Path::new(&search_path).exists() {
        println!("cargo:search={}", search_path);
    } else {
        let search_path = format!("{}/lib64", netcdf.display());
        println!("cargo:search={}", search_path);
    }
}
