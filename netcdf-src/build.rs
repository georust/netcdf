macro_rules! feature {
    ($feature:expr) => {
        std::env::var(concat!("CARGO_FEATURE_", $feature))
    };
}

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    let hdf5_incdir = std::env::var("DEP_HDF5_INCLUDE").unwrap();
    let hdf5_lib = std::env::var("DEP_HDF5_LIBRARY").unwrap();
    let hdf5_hl_lib = std::env::var("DEP_HDF5_HL_LIBRARY").unwrap();

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
        //
        .define("HDF5_C_LIBRARY", &hdf5_lib)
        .define("HDF5_HL_LIBRARY", &hdf5_hl_lib)
        .define("HDF5_INCLUDE_DIR", hdf5_incdir)
        //
        .define("ENABLE_DAP", "OFF") // TODO: feature flag, requires curl
        //
        .profile("RelWithDebInfo"); // TODO: detect opt-level

    if feature!("DAP").is_ok() {
        netcdf_config.define("ENABLE_DAP", "ON");
    }

    let netcdf = netcdf_config.build();

    println!("cargo:lib={}", "netcdf");
    let search_path = format!("{}/lib", netcdf.display());
    if std::path::Path::new(&search_path).exists() {
        println!("cargo:search={}", search_path);
    } else {
        let search_path = format!("{}/lib64", netcdf.display());
        println!("cargo:search={}", search_path);
    }
}
