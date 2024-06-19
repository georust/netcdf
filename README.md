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
* Reading from memory
* Unlimited dimensions
* User defined types, using the feature `derive` (enum, compound, other types requires additional work)

Not (yet) supported (PRs welcome):
* Writing using memory-mapped file

All variable data is read into a contiguous buffer, or into an [ndarray](https://github.com/rust-ndarray/ndarray) if the `ndarray` feature is activated.

## Building

This crate depends on `libnetcdf`, but a static build from source is also supported, which can be enabled using the `static` feature.

The crate is built on several platforms using github actions, and is currently known to build form from source on all major platforms (linux, macos, windows (gnu+msvc)), and through the package installers `conda` and `apt`.

If during compilation there is an error in building the `hdf5` crate, consider using the `static` feature which will include a compatible version of both `netcdf` and `hdf5`. This is likely to be an issue [upstream](https://github.com/aldanor/hdf5-rust/issues/262).

### Building without `libnetcdf` or building statically
1. `git clone https://github.com/georust/netcdf`
2. `git submodule update --init --recursive`
3. `cargo build --features static`


## Documentation

Some examples of usage can be found in the [tests/lib.rs](netcdf/tests/lib.rs) file. The documentation can also be generated using `cargo doc`.


## Thread safety

The `netcdf` crate is thread-safe, although the `netcdf-c` library is not itself threadsafe. To render a safe interface, a global mutex is used to serialize access to the underlying library. Consider using a non threadsafe version of `hdf5` to avoid double locking (performance consideration).

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
