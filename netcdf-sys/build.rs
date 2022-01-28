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
        println!("cargo:includedir={}/include", netcdf_path);
    } else {
        println!("cargo:rerun-if-env-changed=NETCDF_DIR");
        if let Ok(dir) = std::env::var("NETCDF_DIR") {
            println!("cargo:rustc-link-search={}/lib", dir);
            println!("cargo:includedir={}/include", dir);
        } else if let Ok(output) = std::process::Command::new("nc-config")
            .arg("--prefix")
            .output()
        {
            let output = output.stdout;
            let dir = String::from_utf8(output).expect("directory not valid utf8");
            let dir = dir.trim();
            println!("cargo:rustc-link-search={}/lib", dir);
            println!("cargo:includedir={}/include", dir);
        } else {
            // Link to the system netcdf
        }
        println!("cargo:rustc-link-lib=netcdf");
    }

    // Must emit: mmap, memio
}
