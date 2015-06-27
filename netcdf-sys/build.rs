extern crate gcc;
extern crate bindgen;

fn main() {
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let out_dir = std::path::Path::new(&out_dir);
    
    // This is a workaround to avoid using bindgen! macro. Cleaner solution
    // may be available if https://github.com/crabtw/rust-bindgen/issues/201
    // is resolved.
    let rs_path = std::path::Path::new(&out_dir).join("netcdf_bindings.rs");

    let mut bindings = bindgen::builder();
    bindings.forbid_unknown_types();
    // hack for Arch Linux 2015-06-24:
    bindings.clang_arg("-I/usr/lib/clang/3.6.1/include");

    // XXX why do usual clang search paths work for lib but not for include?
    let mnf_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let netcdf_h = std::path::Path::new(&mnf_dir).join(
        "src").join("netcdf_v4.3.3.1.h");
    let netcdf_h  = netcdf_h.to_str().unwrap();
    bindings.header(netcdf_h);
    bindings.link("netcdf");

    let bindings = bindings.generate();
    let bindings = bindings.unwrap();
    bindings.write_to_file(rs_path).unwrap();

    // compile c wrapper to convert CPP constants into proper C types+values
    gcc::compile_library("libncconst.a", &["src/ncconst.c"]);
}
