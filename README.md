# rust-netcdf

[![Build Status](https://travis-ci.org/mhiley/rust-netcdf.svg?branch=master)](https://travis-ci.org/mhiley/rust-netcdf)

High-level [NetCDF](http://www.unidata.ucar.edu/software/netcdf/) bindings for Rust

## Status

Not (yet) supported:

* appending to existing files (using unlimited dimensions),
* user defined types,
* string variables,
* multi-valued attributes,

All variable data is read into a 1-dimensional Vec with the last variable dimension varying fastest.

## Building

rust-netcdf depends on libnetcdf v4.3.3.1

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
let first_val: i32 = var.value_at(&[5, 3]).unwrap();

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

## Documentation

I intend to improve documentation soon. For now, check out [tests/lib.rs](https://github.com/mhiley/rust-netcdf/blob/master/tests/lib.rs) for quite a few usage examples.
