#[derive(netcdf_derive::NcType)]
union NoSupportedUnion {
    A: i32,
    B: u32,
}

fn main() {}
