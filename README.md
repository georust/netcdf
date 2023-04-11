# netcdf

[![Docs](https://docs.rs/netcdf/badge.svg)](https://docs.rs/netcdf)
![Build Status](https://github.com/georust/netcdf/workflows/CI/badge.svg)
[![Crates.io](https://img.shields.io/crates/d/netcdf.svg)](https://crates.io/crates/netcdf)
[![](http://meritbadge.herokuapp.com/netcdf)](https://crates.io/crates/netcdf)
[![codecov](https://codecov.io/gh/georust/netcdf/branch/master/graph/badge.svg)](https://codecov.io/gh/georust/netcdf)
![Crates.io](https://img.shields.io/crates/l/netcdf)
<!-- [![dependency status](https://deps.rs/repo/github/georust/netcdf/status.svg)](https://deps.rs/repo/github/georust/netcdf) -->

Medium-level [netCDF](http://www.unidata.ucar.edu/software/netcdf/) bindings for Rust, allowing easy reading and writing of array-like structures to a file.
netCDF can read and write `hdf5` files, which is a commonly used file format in scientific computing.

## Status

Supported:

* Variables
* Normal Dimensions
* Attributes
* Subgroups
* Open/Append/Create modes
* Reading from memory (read only for now)
* Unlimited dimensions
* string variables
* user defined types (variable length, enum, compound, opaque)

Not (yet) supported:

* some exotic user defined types

All variable data is read into a contiguous buffer, or into an [ndarray](https://github.com/rust-ndarray/rust-ndarray) if the `ndarray` feature is activated.

## Building

This crate depends on `libnetcdf`, but a static build from source is also supported, which can be enabled using the `static` feature.

The crate is built on several platforms using github actions, and is currently known to build form from source on all major platforms (linux, macos, windows (gnu+msvc)), and through the package installers `conda` and `apt`.


## Documentation

Some examples of usage can be found in the [tests/lib.rs](netcdf/tests/lib.rs) file. The documentation can also be generated using `cargo doc`.


## Thread safety

The `netcdf-c` library is not threadsafe. To render a safe interface, a global mutex is used to serialize access to the underlying library. If performance is needed, consider using a non threadsafe version of `hdf5`, so double locking is avoided.

Use of `netcdf-sys` is not thread-safe. Users of this library must take care that calls do not interfere with simultaneous use of e.g. `netcdf`. Using the `hdf5-sys` library could also pose a problem, as this library is used throughout `netCDF-c` and internal state may be disrupted.

## License

Licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.
