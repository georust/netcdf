# rust-netcdf

[![Docs](https://docs.rs/netcdf/badge.svg)](https://docs.rs/netcdf)
[![Build Status](https://travis-ci.org/mhiley/rust-netcdf.svg?branch=master)](https://travis-ci.org/mhiley/rust-netcdf)
[![Crates.io](https://img.shields.io/crates/d/netcdf.svg)](https://crates.io/crates/netcdf)
[![](http://meritbadge.herokuapp.com/netcdf)](https://crates.io/crates/netcdf)
[![Coverage Status](https://coveralls.io/repos/github/mhiley/rust-netcdf/badge.svg?branch=master)](https://coveralls.io/github/mhiley/rust-netcdf?branch=master)
[![codecov](https://codecov.io/gh/mhiley/rust-netcdf/branch/master/graph/badge.svg)](https://codecov.io/gh/mhiley/rust-netcdf)
![Crates.io](https://img.shields.io/crates/l/netcdf)
[![dependency status](https://deps.rs/repo/github/mhiley/rust-netcdf/status.svg)](https://deps.rs/repo/github/mhiley/rust-netcdf)

Medium-level [NetCDF](http://www.unidata.ucar.edu/software/netcdf/) bindings for Rust

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


Not (yet) supported:

* user defined types
* enum types

All variable data is read into a 1-dimensional buffer, with the resulting layout with the last variable varying the fastest.
The data can also be read into an [ndarray](https://github.com/rust-ndarray/rust-ndarray).

## Building

rust-netcdf depends on libnetcdf. The Travis build runs on Ubuntu 16.04 Xenial and installs libnetcdf via apt, which results in netcdf v.4.4.0. rust-netcdf is not widely tested on other versions of netcdf.

You can build the library and run the tests via Docker like this:

```
docker build . -t rust-netcdf
docker run -it --rm rust-netcdf
```

## Documentation

Some examples of usage can be found in the [tests/lib.rs](tests/lib.rs) file. The documentation can also be found using `cargo doc`.

## License

Licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.
