# rust-netcdf

[![Build Status](https://travis-ci.org/mhiley/rust-netcdf.svg?branch=master)](https://travis-ci.org/mhiley/rust-netcdf)

High-level [NetCDF](http://www.unidata.ucar.edu/software/netcdf/) bindings for Rust

## Status

Not (yet) supported:

* appending to existing files (using unlimited dimensions),
* user defined types,
* string variables,
* multi-valued attributes,

All variable data is read into a 1-dimensional Vec with the last variable dimension varying fastest,
or as a [ndarray](https://github.com/bluss/rust-ndarray).

## Building

rust-netcdf depends on libnetcdf v4.3.3.1

You can build the library and run the tests via Docker like this:

```
docker build . -t rust-netcdf
docker run -it --rm rust-netcdf
```

## Read Example

```Rust
// Open file simple_xy.nc:
let file = netcdf::open(&path_to_simple_xy).unwrap();

// Access any variable, attribute, or dimension through simple HashMap's:
let var = file.root.variables.get("data").unwrap();

// Read variable as any NC_TYPE, optionally failing if doing so would
// force a cast:
let data : Vec<i32> = var.get_int(false).unwrap();

// You can also use values() to read the variable, data will be implicitly casted
// if needed
let data : Vec<i32> = var.values().unwrap();

// All variable data is read into 1-dimensional Vec.
for x in 0..(6*12) {
    assert_eq!(data[x], x as i32);
}

// You can also fetch a single value from a dataset,
// using a array slice to index it
let value: i32 = var.value_at(&[5, 3]).unwrap();

// You can also read and fetch values as ArrayD (from the ndarray crate)
let values_array: ArrayD<f64>  = data.as_array().unwrap();
assert_eq!(values_array.shape(),  &[2, 2]);

// subsetted reads are also supported
let values_array: ArrayD<f64>  = data.array_at(&[1, 0], &[2, 3]).unwrap();
assert_eq!(values_array.shape(),  &[2, 3]);

```

## Write Example

```Rust
let f = netcdf::test_file_new("crabs.nc"); // just gets a path inside repo

let mut file = netcdf::create(&f).unwrap();

let dim_name = "ncrabs";
file.root.add_dimension(dim_name, 10).unwrap();

let var_name = "crab_coolness_level";
let data : Vec<i32> = vec![42; 10];
// Variable type written to file is inferred from Vec type:
file.root.add_variable(
            var_name, 
            &vec![dim_name.to_string()],
            &data
        ).unwrap();
```


You can also modify a Variable inside an existing netCDF file, for instance using the previously 
created file :

```Rust
let f = netcdf::test_file_new("crabs.nc"); // get the previously written netCDF file path
// open it in read/write mode
let mut file = netcdf::append(&f).unwrap();
// get a mutable binding of the variable "crab_coolness_level"
let mut var = file.root.variables.get_mut("crab_coolness_level").unwrap();

let data : Vec<i32> = vec![100; 10];
// write 5 first elements of the vector `data` into `var` starting at index 2;
var.put_values_at(&data, &[2], &[5]);
// Change the first value of `var` into '999'
var.put_value_at(999 as f32, &[0]);
```

## Documentation

I intend to improve documentation soon. For now, check out [tests/lib.rs](https://github.com/mhiley/rust-netcdf/blob/master/tests/lib.rs) for quite a few usage examples.

## License

Licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.
