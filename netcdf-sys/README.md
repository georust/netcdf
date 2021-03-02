# netcdf-sys

Rust bindings to `netcdf-c` to locate and link the system libraries neccessary to use `netcdf`.
This library can also build `hdf5` and `netcdf` from source, to allow a fully static linking experience. This is enabled with the `static` feature.

## Detection of netCDF

The detection of `netCDF` has this priority order:
* `static` feature will choose the built static library
* `NETCDF_DIR` environment variable
* `nc-config`
* Default linker-available `netcdf`

If an include file is not found, some features might not be available, despite being included in the library.
