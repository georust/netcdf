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
    println!("cargo::rerun-if-changed=build.rs");

    let hdf5_incdir = std::env::var("DEP_HDF5_INCLUDE").unwrap();
    let mut hdf5_lib = std::env::var("DEP_HDF5_LIBRARY").unwrap();
    let mut hdf5_hl_lib = std::env::var("DEP_HDF5_HL_LIBRARY").unwrap();

    let hdf5_root = format!("{hdf5_incdir}/../");
    #[cfg(unix)]
    {
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
        .define("NETCDF_FIND_SHARED_LIBS", "OFF")
        .define("NETCDF_BUILD_UTILITIES", "OFF")
        .define("NETCDF_ENABLE_EXAMPLES", "OFF")
        .define("NETCDF_ENABLE_DAP_REMOTE_TESTS", "OFF")
        .define("NETCDF_ENABLE_TESTS", "OFF")
        .define("NETCDF_ENABLE_EXTREME_NUMBERS", "OFF")
        .define("NETCDF_ENABLE_PARALLEL_TESTS", "OFF")
        .define("NETCDF_ENABLE_FILTER_TESTING", "OFF")
        .define("ENABLE_BASH_SCRIPT_TESTING", "OFF")
        .define("NETCDF_NETCDF_ENABLE_PLUGINS", "OFF")
        .define("PLUGIN_INSTALL_DIR", "OFF")
        //
        .define("HDF5_ROOT", &hdf5_root)
        //
        .define("NETCDF_ENABLE_LIBXML2", "OFF") // Use bundled xml2
        //
        .define("NETCDF_ENABLE_PARALLEL4", "OFF") // TODO: Enable mpi support
        //
        .define("NETCDF_ENABLE_NCZARR", "OFF") // TODO: requires a bunch of deps
        //
        .define("NETCDF_ENABLE_DAP", "OFF") // TODO: feature flag, requires curl
        .define("NETCDF_ENABLE_BYTERANGE", "OFF") // TODO: feature flag, requires curl
        //
        .profile("RelWithDebInfo"); // TODO: detect opt-level

    let zlib_include_dir = std::env::var("DEP_Z_INCLUDE").unwrap();
    netcdf_config.define("ZLIB_ROOT", format!("{zlib_include_dir}/.."));

    if feature!("DAP").is_ok() {
        netcdf_config.define("NETCDF_ENABLE_DAP", "ON");
        netcdf_config.define("NETCDF_ENABLE_BYTERANGE", "ON");
    }

    if feature!("MPI").is_ok() {
        panic!("MPI feature was requested but the static build of netcdf does not support this");
    }

    let netcdf = netcdf_config.build();

    // Only forward link options to netcdf-sys, so netcdf-sys can
    // optionally choose not to use this build
    println!("cargo::metadata=lib=netcdf");
    let search_path = format!("{}/lib", netcdf.display());
    if std::path::Path::new(&search_path).exists() {
        println!("cargo::metadata=search={search_path}");
    } else {
        let search_path = format!("{}/lib64", netcdf.display());
        println!("cargo::metadata=search={search_path}");
    }
}
