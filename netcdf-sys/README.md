# netcdf-sys

Rust bindings to `netcdf-c` to locate and link the system libraries neccessary to use `netcdf`.
This library can also build `hdf5` and `netcdf` from source, to allow a fully static linking experience. This is enabled with the `static` feature.

## Detection of netCDF

By default the crate uses the system installed `netCDF`. The environment variable `NETCDF_DIR` can be used to use a particular version of `netCDF`. This variable is ignored if compiling from sources.
