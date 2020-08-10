macro_rules! feature {
    ($feature:expr) => {
        std::env::var(concat!("CARGO_FEATURE_", $feature))
    };
}

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    if feature!("STATIC").is_ok() {
        let netcdf_lib = std::env::var("DEP_NETCDFSRC_LIB").unwrap();
        let netcdf_path = std::env::var("DEP_NETCDFSRC_SEARCH").unwrap();

        println!("cargo:rustc-link-lib=static={}", netcdf_lib);
        println!("cargo:rustc-link-search=native={}", netcdf_path);
    } else {
        // Link to the system netcdf
        println!("cargo:rustc-link-lib=netcdf");
    }
}
