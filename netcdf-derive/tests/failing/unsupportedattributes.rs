#[repr(C)]
#[derive(netcdf_derive::NcType)]
struct UnsupportedNetcdfAttribute1 {
    #[netcdf(gibberish)]
    A: i32,
}

#[repr(C)]
#[derive(netcdf_derive::NcType)]
#[netcdf(gibberish)]
struct UnsupportedNetcdfAttribute2 {
    A: i32,
}

#[repr(u8)]
#[derive(netcdf_derive::NcType)]
enum UnSupportedAttribute3 {
    #[netcdf(gibberish)]
    A,
}

fn main() {}
