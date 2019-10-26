#![cfg(test)]

/// Get location of the test files
fn test_location() -> std::path::PathBuf {
    use std::path::Path;

    let mnf_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".into());
    Path::new(&mnf_dir).join("tests").join("testdata")
}

#[test]
/// Use a path to open the netcdf file
fn use_path_to_open() {
    let path = test_location().join("simple_xy.nc");

    let _file = netcdf::open(path).unwrap();
}

#[test]
/// Use a string to open
fn use_string_to_open() {
    let f: String = test_location()
        .join("simple_xy.nc")
        .to_str()
        .unwrap()
        .to_string();
    let _file = netcdf::open(f).unwrap();
}

// Failure tests
#[test]
fn bad_filename() {
    let f = test_location().join("blah_stuff.nc");
    let res_file = netcdf::open(&f);
    assert_eq!(res_file.unwrap_err(), netcdf::error::Error::Netcdf(2));
}

// Read tests
#[test]
fn root_dims() {
    let f = test_location().join("simple_xy.nc");

    let file = netcdf::File::open(&f).unwrap();
    assert_eq!("simple_xy.nc", file.name());

    assert_eq!(file.root().dimension("x").unwrap().len(), 6);
    assert_eq!(file.root().dimension("y").unwrap().len(), 12);
}

#[test]
fn access_through_deref() {
    let f = test_location().join("simple_xy.nc");

    let file = netcdf::File::open(&f).unwrap();

    assert_eq!(file.dimension("x").unwrap().len(), 6);
    assert_eq!(file.dimension("y").unwrap().len(), 12);

    let d = tempfile::tempdir().unwrap();
    let f = d.path().join("derefmut.nc");
    let mut file = netcdf::create(&f).unwrap();

    file.add_dimension("time", 10).unwrap();

    assert_eq!(
        file.dimension("time")
            .expect("Could not find dimension")
            .len(),
        10
    );
}

#[test]
fn global_attrs() {
    use netcdf::attribute::AttrValue;
    let f = test_location().join("patmosx_v05r03-preliminary_NOAA-19_asc_d20130630_c20140325.nc");

    let file = netcdf::File::open(&f).unwrap();

    let ch1_attr = &file
        .root()
        .attribute("CH1_DARK_COUNT")
        .expect("Could not find attribute");
    let chi = ch1_attr.value().unwrap();
    let eps = 1e-6;
    if let AttrValue::Float(x) = chi {
        assert!((x - 40.65863).abs() < eps);
    } else {
        panic!("Did not get the expected attr type");
    }

    let sensor_attr = &file
        .root()
        .attribute("sensor")
        .expect("Could not find attribute");
    let sensor_data = sensor_attr.value().unwrap();
    if let AttrValue::Str(x) = sensor_data {
        assert_eq!("AVHRR/3", x);
    } else {
        panic!("Did not get the expected attr type");
    }
}

#[test]
fn var_as_different_types() {
    let f = test_location().join("simple_xy.nc");

    let file = netcdf::File::open(&f).unwrap();

    let mut data = vec![0; 6 * 12];
    let var = &file
        .root()
        .variable("data")
        .expect("Could not find variable");
    var.values_to(&mut data, None, None).unwrap();

    for (x, d) in data.iter().enumerate() {
        assert_eq!(*d, x as i32);
    }

    // do the same thing but cast to float
    let mut data = vec![0.0; 6 * 12];
    var.values_to(&mut data, None, None).unwrap();

    for (x, d) in data.iter().enumerate() {
        assert!((*d - x as f32).abs() < 1e-5);
    }
}

#[test]
fn test_index_fetch() {
    let f = test_location().join("simple_xy.nc");

    let file = netcdf::File::open(&f).unwrap();

    let var = &file
        .root()
        .variable("data")
        .expect("Could not find variable");
    // Gets first value
    let first_val: i32 = var.value(None).unwrap();
    let other_val: i32 = var.value(Some(&[5, 3])).unwrap();

    assert_eq!(first_val, 0 as i32);
    assert_eq!(other_val, 63 as i32);
}

#[test]
#[cfg(feature = "ndarray")]
fn last_dim_varies_fastest() {
    let f = test_location().join("simple_xy.nc");

    let file = netcdf::File::open(&f).unwrap();

    let var = &file
        .root()
        .variable("data")
        .expect("Could not find variable");
    let data = var.values::<i32>(None, None).unwrap();

    let nx = var.dimensions()[0].len();
    let ny = var.dimensions()[1].len();

    assert_eq!(nx, 6);
    assert_eq!(ny, 12);
    assert_eq!(nx * ny, data.len());

    for x in 0..nx {
        for y in 0..ny {
            let ind = x * ny + y;
            assert_eq!(data.as_slice().unwrap()[ind], ind as i32);
        }
    }
}

#[test]
fn attributes_read() {
    let f = test_location().join("patmosx_v05r03-preliminary_NOAA-19_asc_d20130630_c20140325.nc");
    let file = netcdf::open(&f).unwrap();

    let attr = &file
        .attribute("PROGLANG")
        .expect("Could not find attribute");

    assert_eq!(attr.name(), "PROGLANG");

    for attr in file.attributes() {
        let _val = attr.value().expect("Could not get value");
    }

    let d = tempfile::tempdir().expect("Could not get tempdir");
    let path = d.path().join("attributes_read.nc");
    let mut file = netcdf::create(path).expect("Could not create file");

    let var = &mut file
        .add_variable::<f32>("var", &[])
        .expect("Could not add variable");
    var.add_attribute("att", "some attribute")
        .expect("Could not add attribute");
    assert_eq!(var.vartype(), netcdf_sys::NC_FLOAT);

    for attr in var.attributes() {
        attr.value().unwrap();
    }
}

#[test]
fn dimension_lengths() {
    let d = tempfile::tempdir().expect("Could not create tempdir");
    let path = d.path().join("dimension_lengths");
    let mut file = netcdf::create(path).expect("Could not create file");

    file.add_unlimited_dimension("unlim")
        .expect("Could not create dimension");
    file.add_dimension("lim", 10)
        .expect("Could not create dimension");

    let dim = &file.dimension("unlim").expect("Could not find dim");
    assert_eq!(dim.len(), 0);

    let dim = &file.dimension("lim").expect("Could not find dim");
    assert_eq!(dim.len(), 10);

    for dim in file.dimensions() {
        assert!(dim.len() == 0 || dim.len() == 10);
    }
}

#[test]
fn netcdf_error() {
    let path = ".";
    let err = netcdf::open(path).unwrap_err();

    use std::error::Error;
    println!("{} {:?}", err, err.source());

    let err: netcdf::error::Error = "hello".into();
    println!("{}", err);
    let err: netcdf::error::Error = String::from("hello").into();
    println!("{}", err);

    let d = tempfile::tempdir().expect("Could not get tempdir");
    let path = d.path().join("netcdf_error.nc");
    let mut file = netcdf::create(path).expect("Could not create file");

    file.add_variable::<i8>("var", &["v"]).unwrap_err();
    file.add_dimension("v", 3).expect("Could not add dimension");
    file.add_variable::<i8>("var", &["v"]).unwrap();
    file.add_variable::<i8>("var", &["v"]).unwrap_err();
}

#[test]
#[cfg(feature = "ndarray")]
fn open_pres_temp_4d() {
    use netcdf::attribute::AttrValue;
    let f = test_location().join("pres_temp_4D.nc");

    let file = netcdf::File::open(&f).unwrap();

    let pres = &file.root().variable("pressure").unwrap();
    assert_eq!(pres.dimensions()[0].name(), "time");
    assert_eq!(pres.dimensions()[1].name(), "level");
    assert_eq!(pres.dimensions()[2].name(), "latitude");
    assert_eq!(pres.dimensions()[3].name(), "longitude");

    // test var attributes
    assert_eq!(
        pres.attribute("units")
            .expect("Could not find attribute")
            .value()
            .unwrap(),
        AttrValue::Str("hPa".to_string())
    );
}

#[test]
#[cfg(feature = "ndarray")]
fn ndarray_read_with_indices() {
    let f = test_location().join("pres_temp_4D.nc");
    let file = netcdf::open(f).unwrap();

    let var = &file.variable("pressure").unwrap();

    let sizes = [
        var.dimensions()[0].len(),
        var.dimensions()[1].len(),
        1,
        var.dimensions()[2].len(),
    ];
    let indices = [0, 0, 3, 0];
    let values = var.values::<f32>(Some(&indices), Some(&sizes)).unwrap();

    assert_eq!(values.shape(), sizes);

    let indices = [0, 1, 3, 0];
    let sizes = [
        var.dimensions()[0].len(),
        var.dimensions()[1].len() - 1,
        2,
        var.dimensions()[2].len(),
    ];
    let values = var.values::<f32>(Some(&indices), Some(&sizes)).unwrap();
    assert_eq!(values.shape(), sizes);
}

#[test]
fn nc4_groups() {
    let f = test_location().join("simple_nc4.nc");

    let file = netcdf::File::open(&f).unwrap();

    let grp1 = &file.group("grp1").expect("Could not find group");
    assert_eq!(grp1.name(), "grp1");

    let mut data = vec![0i32; 6 * 12];
    let var = &grp1.variable("data").unwrap();
    var.values_to(&mut data, None, None).unwrap();
    for (i, x) in data.iter().enumerate() {
        assert_eq!(*x, i as i32);
    }
}

#[test]
fn create_group_dimensions() {
    let d = tempfile::tempdir().unwrap();
    let filepath = d.path().join("create_group.nc");
    let mut f = netcdf::create(filepath).unwrap();

    f.add_dimension("x", 20).unwrap();

    let g = &mut f.add_group("gp1").unwrap();

    g.add_dimension("x", 100).unwrap();
    g.add_variable::<u8>("y", &["x"]).unwrap();

    let gg = &mut g.add_group("gp2").unwrap();
    gg.add_variable::<i8>("y", &["x"]).unwrap();

    gg.add_dimension("x", 30).unwrap();
    gg.add_variable::<i8>("z", &["x"]).unwrap();

    assert_eq!(
        f.group("gp1")
            .expect("Could not find group")
            .variable("y")
            .unwrap()
            .dimensions()[0]
            .len(),
        100
    );
    assert_eq!(
        f.group("gp1")
            .expect("Could not find group")
            .group("gp2")
            .expect("Could not find group")
            .variable("y")
            .unwrap()
            .dimensions()[0]
            .len(),
        100
    );
    assert_eq!(
        f.group("gp1")
            .expect("Could not find group")
            .group("gp2")
            .expect("Could not find group")
            .variable("z")
            .expect("Could not find variable")
            .dimensions()[0]
            .len(),
        30
    );
}

// Write tests
#[test]
fn create() {
    let d = tempfile::tempdir().unwrap();
    let f = d.path().join("create.nc");

    let file = netcdf::File::create(&f).unwrap();
    assert_eq!("create.nc", file.name());
}

#[test]
#[cfg(feature = "ndarray")]
fn def_dims_vars_attrs() {
    let d = tempfile::tempdir().unwrap();
    {
        let f = d.path().join("def_dims_vars_attrs.nc");

        let mut file = netcdf::File::create(&f).unwrap();

        let dim1_name = "ljkdsjkldfs";
        let dim2_name = "dsfkdfskl";
        file.root_mut().add_dimension(dim1_name, 10).unwrap();
        file.root_mut().add_dimension(dim2_name, 20).unwrap();
        assert_eq!(
            file.root()
                .dimension(dim1_name)
                .expect("Could not find dimension")
                .len(),
            10
        );
        assert_eq!(
            file.root()
                .dimension(dim2_name)
                .expect("Could not find dimension")
                .len(),
            20
        );

        let var_name = "varstuff_int";
        let data: Vec<i32> = vec![42; 10 * 20];
        let var = &mut file
            .root_mut()
            .add_variable::<i32>(var_name, &[dim1_name, dim2_name])
            .unwrap();
        var.put_values(data.as_slice(), None, None).unwrap();
        assert_eq!(var.dimensions()[0].len(), 10);
        assert_eq!(var.dimensions()[1].len(), 20);

        let var_name = "varstuff_float";
        let data: Vec<f32> = vec![42.2; 10];
        let var = &mut file
            .root_mut()
            .add_variable::<f32>(var_name, &[dim1_name])
            .unwrap();
        var.put_values(data.as_slice(), None, None).unwrap();
        assert_eq!(var.dimensions()[0].len(), 10);

        // test global attrs
        file.root_mut().add_attribute("testattr1", 3).unwrap();
        file.root_mut()
            .add_attribute("testattr2", "Global string attr".to_string())
            .unwrap();

        // test var attrs
        let var = file.root_mut().variable_mut(var_name).unwrap();
        var.add_attribute("varattr1", 5).unwrap();
        var.add_attribute("varattr2", "Variable string attr".to_string())
            .unwrap();
    }

    // now, read in the file we created and verify everything
    {
        use ndarray::ArrayD;
        let f = d.path().join("def_dims_vars_attrs.nc");

        let file = netcdf::File::open(&f).unwrap();

        // verify dimensions
        let dim1_name = "ljkdsjkldfs";
        let dim2_name = "dsfkdfskl";
        let dim1 = &file
            .root()
            .dimension(dim1_name)
            .expect("Could not find dimension");
        let dim2 = &file
            .root()
            .dimension(dim2_name)
            .expect("Could not find dimension");
        assert_eq!(dim1.len(), 10);
        assert_eq!(dim2.len(), 20);

        // verify variable data
        let var_name = "varstuff_int";
        let data_test: ArrayD<i32> = ArrayD::from_elem(ndarray::IxDyn(&[10, 20]), 42i32);
        let data_file = file
            .root()
            .variable(var_name)
            .expect("Could not find variable")
            .values::<i32>(None, None)
            .unwrap();
        assert_eq!(data_test.len(), data_file.len());
        assert_eq!(data_test, data_file);

        let var_name = "varstuff_float";
        let data_test = ArrayD::from_elem(ndarray::IxDyn(&[10]), 42.2f32);
        let data_file = file
            .root()
            .variable(var_name)
            .expect("Could not find variable")
            .values::<f32>(None, None)
            .unwrap();
        assert_eq!(data_test, data_file);

        // verify global attrs
        use netcdf::attribute::AttrValue;
        assert_eq!(
            AttrValue::Int(3),
            file.root()
                .attribute("testattr1")
                .expect("Could not find attribute")
                .value()
                .unwrap()
        );
        assert_eq!(
            AttrValue::Str("Global string attr".into()),
            file.root()
                .attribute("testattr2")
                .expect("Could not find attribute")
                .value()
                .unwrap()
        );

        // verify var attrs
        assert_eq!(
            AttrValue::Int(5),
            file.root()
                .variable(var_name)
                .expect("Could not find variable")
                .attribute("varattr1")
                .expect("Could not find attribute")
                .value()
                .unwrap()
        );
        assert_eq!(
            AttrValue::Str("Variable string attr".into()),
            file.root()
                .variable(var_name)
                .expect("Could not find variable")
                .attribute("varattr2")
                .expect("Could not find attribute")
                .value()
                .unwrap()
        );
    }
}

#[test]
fn all_var_types() {
    // write
    let d = tempfile::tempdir().unwrap();
    let name = "all_var_types.nc";
    {
        let f = d.path().join(name);
        let mut file = netcdf::File::create(&f).unwrap();

        let dim_name = "dim1";

        let root = file.root_mut();
        root.add_dimension(dim_name, 10).unwrap();

        // byte
        let data = vec![42i8; 10];
        let var_name = "var_byte";
        let var = root.add_variable::<i8>(var_name, &[dim_name]).unwrap();
        var.put_values(&data, None, None).unwrap();

        let data = vec![42u8; 10];
        let var_name = "var_char";
        let var = root.add_variable::<u8>(var_name, &[dim_name]).unwrap();
        var.put_values(&data, None, None).unwrap();

        // short
        let data = vec![42i16; 10];
        let var_name = "var_short";
        let var = root.add_variable::<i16>(var_name, &[dim_name]).unwrap();
        var.put_values(&data, None, None).unwrap();

        // ushort
        let data = vec![42u16; 10];
        let var_name = "var_ushort";
        let var = root.add_variable::<u16>(var_name, &[dim_name]).unwrap();
        var.put_values(&data, None, None).unwrap();

        // int
        let data = vec![42i32; 10];
        let var_name = "var_int";
        let var = root.add_variable::<i32>(var_name, &[dim_name]).unwrap();
        var.put_values(&data, None, None).unwrap();

        // uint
        let data = vec![42u32; 10];
        let var_name = "var_uint";
        let var = root.add_variable::<u32>(var_name, &[dim_name]).unwrap();
        var.put_values(&data, None, None).unwrap();

        // int64
        let data = vec![42i64; 10];
        let var_name = "var_int64";
        let var = root.add_variable::<i64>(var_name, &[dim_name]).unwrap();
        var.put_values(&data, None, None).unwrap();

        // uint64
        let data = vec![42u64; 10];
        let var_name = "var_uint64";
        let var = root.add_variable::<u64>(var_name, &[dim_name]).unwrap();
        var.put_values(&data, None, None).unwrap();

        // float
        let data = vec![42.2f32; 10];
        let var_name = "var_float";
        let var = root.add_variable::<f32>(var_name, &[dim_name]).unwrap();
        var.put_values(&data, None, None).unwrap();

        // double
        let data = vec![42.2f64; 10];
        let var_name = "var_double";
        let var = root.add_variable::<f64>(var_name, &[dim_name]).unwrap();
        var.put_values(&data, None, None).unwrap();
    }

    {
        // read
        let f = d.path().join(name);
        let file = netcdf::open(f).unwrap();

        //byte
        let mut data = vec![0i8; 10];
        file.root()
            .variable("var_byte")
            .unwrap()
            .values_to(&mut data, None, None)
            .unwrap();
        assert_eq!(vec![42i8; 10], data);

        // ubyte
        let mut data = vec![0u8; 10];
        file.root()
            .variable("var_char")
            .unwrap()
            .values_to(&mut data, None, None)
            .unwrap();
        assert_eq!(vec![42u8; 10], data);

        // short
        let mut data = vec![0i16; 10];
        file.root()
            .variable("var_short")
            .unwrap()
            .values_to(&mut data, None, None)
            .unwrap();
        assert_eq!(vec![42i16; 10], data);

        // ushort
        let mut data = vec![0u16; 10];
        file.root()
            .variable("var_ushort")
            .unwrap()
            .values_to(&mut data, None, None)
            .unwrap();
        assert_eq!(vec![42u16; 10], data);

        // int
        let mut data = vec![0i32; 10];
        file.root()
            .variable("var_int")
            .unwrap()
            .values_to(&mut data, None, None)
            .unwrap();
        assert_eq!(vec![42i32; 10], data);

        // uint
        let mut data = vec![0u32; 10];
        file.root()
            .variable("var_uint")
            .unwrap()
            .values_to(&mut data, None, None)
            .unwrap();
        assert_eq!(vec![42u32; 10], data);

        // int64
        let mut data = vec![0i64; 10];
        file.root()
            .variable("var_int64")
            .unwrap()
            .values_to(&mut data, None, None)
            .unwrap();
        assert_eq!(vec![42i64; 10], data);

        // uint64
        let mut data = vec![0u64; 10];
        file.root()
            .variable("var_uint64")
            .unwrap()
            .values_to(&mut data, None, None)
            .unwrap();
        assert_eq!(vec![42u64; 10], data);

        // float
        let mut data = vec![0.0f32; 10];
        file.root()
            .variable("var_float")
            .unwrap()
            .values_to(&mut data, None, None)
            .unwrap();
        assert_eq!(vec![42.2f32; 10], data);

        // double
        let mut data = vec![0.0f64; 10];
        file.root()
            .variable("var_double")
            .unwrap()
            .values_to(&mut data, None, None)
            .unwrap();
        assert_eq!(vec![42.2f64; 10], data);
    }
}

#[test]
fn all_attr_types() {
    let d = tempfile::tempdir().unwrap();
    let u8string = "Testing utf8 with æøå and even 😀";
    {
        let f = d.path().join("all_attr_types.nc");
        let mut file = netcdf::File::create(&f).unwrap();

        file.root_mut().add_attribute("attr_byte", 3 as i8).unwrap();
        file.root_mut()
            .add_attribute("attr_ubyte", 3 as u8)
            .unwrap();
        file.root_mut()
            .add_attribute("attr_short", 3 as i16)
            .unwrap();
        file.root_mut()
            .add_attribute("attr_ushort", 3 as u16)
            .unwrap();
        file.root_mut().add_attribute("attr_int", 3 as i32).unwrap();
        file.root_mut()
            .add_attribute("attr_uint", 3 as u32)
            .unwrap();
        file.root_mut()
            .add_attribute("attr_int64", 3 as i64)
            .unwrap();
        file.root_mut()
            .add_attribute("attr_uint64", 3 as u64)
            .unwrap();
        file.root_mut()
            .add_attribute("attr_float", 3.2 as f32)
            .unwrap();
        file.root_mut()
            .add_attribute("attr_double", 3.2 as f64)
            .unwrap();
        file.root_mut()
            .add_attribute("attr_text", "Hello world!")
            .unwrap();

        file.root_mut()
            .add_attribute("attr_text_utf8", u8string)
            .unwrap();
    }

    {
        use netcdf::attribute::AttrValue;
        let f = d.path().join("all_attr_types.nc");
        let file = netcdf::File::open(&f).unwrap();

        assert_eq!(
            AttrValue::Uchar(3),
            file.root()
                .attribute("attr_ubyte")
                .unwrap()
                .value()
                .unwrap()
        );
        assert_eq!(
            AttrValue::Schar(3),
            file.root().attribute("attr_byte").unwrap().value().unwrap()
        );
        assert_eq!(
            AttrValue::Ushort(3),
            file.root()
                .attribute("attr_ushort")
                .unwrap()
                .value()
                .unwrap()
        );
        assert_eq!(
            AttrValue::Short(3),
            file.root()
                .attribute("attr_short")
                .unwrap()
                .value()
                .unwrap()
        );
        assert_eq!(
            AttrValue::Int(3),
            file.root().attribute("attr_int").unwrap().value().unwrap()
        );
        assert_eq!(
            AttrValue::Uint(3),
            file.root().attribute("attr_uint").unwrap().value().unwrap()
        );
        assert_eq!(
            AttrValue::Ulonglong(3),
            file.root()
                .attribute("attr_uint64")
                .unwrap()
                .value()
                .unwrap()
        );
        assert_eq!(
            AttrValue::Longlong(3),
            file.root()
                .attribute("attr_int64")
                .unwrap()
                .value()
                .unwrap()
        );
        assert_eq!(
            AttrValue::Float(3.2),
            file.root()
                .attribute("attr_float")
                .unwrap()
                .value()
                .unwrap()
        );
        assert_eq!(
            AttrValue::Double(3.2),
            file.root()
                .attribute("attr_double")
                .unwrap()
                .value()
                .unwrap()
        );
        assert_eq!(
            AttrValue::Str("Hello world!".into()),
            file.root().attribute("attr_text").unwrap().value().unwrap()
        );
        assert_eq!(
            AttrValue::Str(u8string.into()),
            file.root()
                .attribute("attr_text_utf8")
                .unwrap()
                .value()
                .unwrap()
        );
    }
}

#[test]
#[cfg(feature = "ndarray")]
/// Tests the shape of a variable
/// when fetched using "Variable::as_array()"
fn fetch_ndarray() {
    let f = test_location().join("pres_temp_4D.nc");
    let file = netcdf::File::open(&f).unwrap();

    let pres = &file
        .root()
        .variable("pressure")
        .expect("Could not find variable");
    let values_array = pres.values::<f64>(None, None).unwrap();
    assert_eq!(values_array.shape(), &[2, 2, 6, 12]);
}

#[test]
// test file modification
fn append() {
    let d = tempfile::tempdir().unwrap();
    let f = d.path().join("append.nc");
    let dim_name = "some_dimension";
    {
        // first creates a simple netCDF file
        // and create a variable called "some_variable" in it
        let mut file_w = netcdf::File::create(&f).unwrap();
        file_w.root_mut().add_dimension(dim_name, 3).unwrap();
        let var = &mut file_w
            .root_mut()
            .add_variable::<i32>("some_variable", &[dim_name])
            .unwrap();
        var.put_values::<i32>(&[1, 2, 3], None, None).unwrap();
        // close it (done when `file_w` goes out of scope)
    }
    {
        // re-open it in append mode
        // and create a variable called "some_other_variable"
        let mut file_a = netcdf::append(&f).unwrap();
        let var = &mut file_a
            .root_mut()
            .add_variable::<i32>("some_other_variable", &[dim_name])
            .unwrap();
        var.put_values::<i32>(&[4, 5, 6], None, None).unwrap();
        // close it (done when `file_a` goes out of scope)
    }
    // finally open  the file in read only mode
    // and test the existence of both variable
    let file = netcdf::open(&f).unwrap();
    assert!(file
        .root()
        .variables()
        .find(|x| x.name() == "some_variable")
        .is_some());
    assert!(file
        .root()
        .variables()
        .find(|x| x.name() == "some_other_variable")
        .is_some());
}

#[test]
// test file modification
fn put_single_value() {
    let d = tempfile::tempdir().unwrap();
    let f = d.path().join("append_value.nc");
    let dim_name = "some_dimension";
    let var_name = "some_variable";
    {
        // first creates a simple netCDF file
        // and create a variable called "some_variable" in it
        let mut file_w = netcdf::File::create(&f).unwrap();
        file_w.root_mut().add_dimension(dim_name, 3).unwrap();
        let var = &mut file_w
            .root_mut()
            .add_variable::<f32>(var_name, &[dim_name])
            .unwrap();
        var.put_values(&[1., 2., 3.], None, None).unwrap();
    }
    let indices: [usize; 1] = [0];
    {
        // re-open it in append mode
        let mut file_a = netcdf::append(&f).unwrap();
        let var = &mut file_a.root_mut().variable_mut(var_name).unwrap();
        var.put_value(100.0f32, Some(&indices)).unwrap();
        // close it (done when `file_a` goes out of scope)
    }
    // finally open  the file in read only mode
    // and test the values of 'some_variable'
    let file = netcdf::File::open(&f).unwrap();
    let var = &file
        .root()
        .variable(var_name)
        .expect("Could not find variable");
    assert_eq!(var.value(Some(&indices)), Ok(100.0));
}

#[test]
// test file modification
fn put_values() {
    let d = tempfile::tempdir().unwrap();
    let f = d.path().join("append_values.nc");
    let dim_name = "some_dimension";
    let var_name = "some_variable";
    {
        // first creates a simple netCDF file
        // and create a variable called "some_variable" in it
        let mut file_w = netcdf::File::create(&f).unwrap();
        file_w.root_mut().add_dimension(dim_name, 3).unwrap();
        let var = &mut file_w
            .root_mut()
            .add_variable::<i32>(var_name, &[dim_name])
            .unwrap();
        var.put_values(&[1i32, 2, 3], None, None).unwrap();
        // close it (done when `file_w` goes out of scope)
    }
    let indices = &[1];
    let values = &[100i32, 200];
    let len = &[values.len()];
    {
        // re-open it in append mode
        let mut file_a = netcdf::append(&f).unwrap();
        let var = &mut file_a.root_mut().variable_mut(var_name).unwrap();
        let res = var.put_values(values, Some(indices), Some(len));
        assert_eq!(res, Ok(()));
        // close it (done when `file_a` goes out of scope)
    }
    // finally open  the file in read only mode
    // and test the values of 'some_variable'
    let file = netcdf::File::open(&f).unwrap();
    let var = &file
        .root()
        .variable(var_name)
        .expect("Could not find variable");
    let mut d = vec![0i32; 3];
    var.values_to(d.as_mut_slice(), None, None).unwrap();
    assert_eq!(d, [1, 100, 200]);
}

#[test]
/// Test setting a fill value when creating a Variable
fn set_fill_value() {
    let d = tempfile::tempdir().unwrap();
    let f = d.path().join("fill_value.nc");
    let dim_name = "some_dimension";
    let var_name = "some_variable";
    let fill_value = -2 as i32;

    let mut file_w = netcdf::File::create(&f).unwrap();
    file_w.root_mut().add_dimension(dim_name, 3).unwrap();
    let var = &mut file_w
        .root_mut()
        .add_variable::<i32>(var_name, &[dim_name])
        .unwrap();
    var.set_fill_value(fill_value).unwrap();

    var.put_values(&[2, 3], Some(&[1]), None).unwrap();

    let mut rvar = [0i32; 3];
    var.values_to(&mut rvar, None, None).unwrap();

    assert_eq!(rvar, [fill_value, 2, 3]);

    let var = &file_w
        .root()
        .variable(var_name)
        .expect("Could not find variable");
    let attr = var
        .attribute("_FillValue")
        .expect("could not find attribute")
        .value()
        .unwrap();
    // compare requested fill_value and attribute _FillValue
    use netcdf::attribute::AttrValue;
    assert_eq!(AttrValue::Int(fill_value), attr);

    let fill = var.fill_value::<i32>().unwrap();
    assert_eq!(fill, Some(fill_value));

    // Expecting an error when trying to get the wrong variable type
    var.fill_value::<f32>().unwrap_err();
}

#[test]
/// Test reading a slice of a variable into a buffer
fn read_slice_into_buffer() {
    let f = test_location().join("simple_xy.nc");
    let file = netcdf::File::open(&f).unwrap();
    let pres = &file
        .root()
        .variable("data")
        .expect("Could not find variable");
    // pre-allocate the Array
    let mut values = vec![0i8; 6 * 3];
    let ind = &[0, 0];
    let len = &[6, 3];
    pres.values_to(values.as_mut_slice(), Some(ind), Some(len))
        .unwrap();
    let expected_values = [
        0i8, 1, 2, 12, 13, 14, 24, 25, 26, 36, 37, 38, 48, 49, 50, 60, 61, 62,
    ];
    for i in 0..values.len() {
        assert_eq!(expected_values[i], values[i]);
    }
}

#[test]
#[should_panic]
fn read_mismatched() {
    let f = test_location().join("simple_xy.nc");
    let file = netcdf::open(f).unwrap();

    let pres = &file.root().variable("data").expect("variable not found");

    let mut d = vec![0; 40];
    pres.values_to(d.as_mut_slice(), None, Some(&[40, 1]))
        .unwrap();
}

#[test]
fn use_compression_chunking() {
    let d = tempfile::tempdir().unwrap();
    let f = d.path().join("compressed_var.nc");
    let mut file = netcdf::create(f).unwrap();

    file.root_mut().add_dimension("x", 10).unwrap();

    let var = &mut file
        .root_mut()
        .add_variable::<i32>("compressed", &["x"])
        .unwrap();
    var.compression(5).unwrap();
    var.chunking(&[5]).unwrap();

    let v = vec![0i32; 10];
    var.put_values(&v, None, None).unwrap();

    let var = &mut file
        .add_variable::<i32>("compressed2", &["x", "x"])
        .unwrap();
    var.compression(9).unwrap();
    var.chunking(&[5, 5]).unwrap();
    var.put_values(&[1i32, 2, 3, 4, 5, 6, 7, 8, 9, 10], None, Some(&[10, 1]))
        .unwrap();

    let var = &mut file.add_variable::<i32>("chunked3", &["x"]).unwrap();
    assert_eq!(
        var.chunking(&[2, 2]).unwrap_err(),
        netcdf::error::Error::SliceLen
    );

    file.add_dimension("y", 0).unwrap();
    let var = &mut file.add_variable::<u8>("chunked4", &["y", "x"]).unwrap();

    var.chunking(&[100, 2]).unwrap();
}

#[test]
fn set_compression_all_variables_in_a_group() {
    let d = tempfile::tempdir().expect("Could not create tempdir");
    let path = d.path().join("set_compression_all_variables_in_a_group.nc");
    let mut file = netcdf::create(path).expect("Could not create file");

    file.add_dimension("x", 10)
        .expect("Could not create dimension");
    file.add_dimension("y", 15)
        .expect("Could not create dimension");
    file.add_variable::<u8>("var0", &["x", "y"])
        .expect("Could not create variable");
    file.add_variable::<u8>("var1", &["x", "y"])
        .expect("Could not create variable");
    file.add_variable::<u8>("var2", &["x", "y"])
        .expect("Could not create variable");
    file.add_variable::<u8>("var3", &["x", "y"])
        .expect("Could not create variable");

    for ref mut var in file.variables_mut() {
        var.compression(9).expect("Could not set compression level");
    }

    let var = file.variable_mut("var0").unwrap();
    var.compression(netcdf_sys::NC_MAX_DEFLATE_LEVEL + 1)
        .unwrap_err();
}

#[test]
#[cfg(feature = "memory")]
fn read_from_memory() {
    use std::io::Read;
    let origfile = test_location().join("simple_xy.nc");
    let mut origfile = std::fs::File::open(origfile).unwrap();
    let mut bytes = Vec::new();
    origfile.read_to_end(&mut bytes).unwrap();

    let file = netcdf::open_mem(None, &bytes).unwrap();
    let x = &(*file).root().dimension("x").unwrap();
    assert_eq!(x.len(), 6);
    let y = &(*file).root().dimension("y").unwrap();
    assert_eq!(y.len(), 12);
    let mut v = vec![0i32; 6 * 12];
    (*file)
        .root()
        .variable("data")
        .expect("Could not find variable")
        .values_to(&mut v, None, None)
        .unwrap();
    for (i, v) in v.iter().enumerate() {
        assert_eq!(*v, i as _);
    }
}

#[test]
fn add_confliciting_dimensions() {
    let d = tempfile::tempdir().unwrap();

    let mut file = netcdf::create(d.path().join("conflict_dim.nc")).unwrap();

    file.add_dimension("x", 10).unwrap();
    let e = file.add_dimension("x", 11).unwrap_err();
    assert_eq!(
        e,
        netcdf::error::Error::AlreadyExists("dimension".to_string())
    );
    assert_eq!(file.dimension("x").unwrap().len(), 10);
}

#[test]
fn add_conflicting_variables() {
    let d = tempfile::tempdir().unwrap();
    let mut file = netcdf::create(d.path().join("conflict_var")).unwrap();

    file.add_dimension("x", 10).unwrap();
    file.add_dimension("y", 20).unwrap();

    file.add_variable::<i32>("x", &["x"]).unwrap();

    let e = file.add_variable::<f32>("x", &["y"]).unwrap_err();
    assert_eq!(
        e,
        netcdf::error::Error::AlreadyExists("variable".to_string())
    );
    assert_eq!(10, file.variable("x").unwrap().dimensions()[0].len());
}

#[test]
fn unlimited_dimension_single_putting() {
    let d = tempfile::tempdir().unwrap();
    let mut file = netcdf::create(d.path().join("unlim_single.nc")).unwrap();

    file.add_unlimited_dimension("x").unwrap();
    file.add_unlimited_dimension("y").unwrap();

    let var = &mut file.add_variable::<u8>("var", &["x", "y"]).unwrap();
    var.set_fill_value(0u8).unwrap();

    var.put_value(1, None).unwrap();
    assert_eq!(var.dimensions()[0].len(), 1);
    assert_eq!(var.dimensions()[1].len(), 1);
    var.put_value(2, Some(&[0, 1])).unwrap();
    assert_eq!(var.dimensions()[0].len(), 1);
    assert_eq!(var.dimensions()[1].len(), 2);
    var.put_value(3, Some(&[2, 0])).unwrap();
    assert_eq!(var.dimensions()[0].len(), 3);
    assert_eq!(var.dimensions()[1].len(), 2);

    let mut v = vec![0; 6];
    var.values_to(&mut v, None, Some(&[3, 2])).unwrap();

    assert_eq!(v, &[1, 2, 0, 0, 3, 0]);
}

fn check_equal<T>(var: &netcdf::Variable, check: &[T])
where
    T: netcdf::variable::Numeric
        + std::clone::Clone
        + std::default::Default
        + std::fmt::Debug
        + std::cmp::PartialEq,
{
    let mut v: Vec<T> = vec![Default::default(); check.len()];
    var.values_to(&mut v, None, None).unwrap();
    assert_eq!(v.as_slice(), check);
}

#[test]
fn unlimited_dimension_multi_putting() {
    let d = tempfile::tempdir().unwrap();
    let mut file = netcdf::create(d.path().join("unlim_multi.nc")).unwrap();

    file.add_unlimited_dimension("x").unwrap();
    file.add_unlimited_dimension("y").unwrap();
    file.add_dimension("z", 2).unwrap();
    file.add_unlimited_dimension("x2").unwrap();
    file.add_unlimited_dimension("x3").unwrap();
    file.add_unlimited_dimension("x4").unwrap();

    let var = &mut file.add_variable::<u8>("one_unlim", &["x", "z"]).unwrap();
    var.put_values(&[0u8, 1, 2, 3], None, None).unwrap();
    check_equal(var, &[0u8, 1, 2, 3]);
    var.put_values(&[0u8, 1, 2, 3, 4, 5, 6, 7], None, None)
        .unwrap();
    check_equal(var, &[0u8, 1, 2, 3, 4, 5, 6, 7]);

    let var = &mut file
        .add_variable::<u8>("unlim_first", &["z", "x2"])
        .unwrap();
    var.put_values(&[0u8, 1, 2, 3], None, None).unwrap();
    check_equal(var, &[0u8, 1, 2, 3]);
    var.put_values(&[0u8, 1, 2, 3, 4, 5, 6, 7], None, None)
        .unwrap();
    check_equal(var, &[0u8, 1, 2, 3, 4, 5, 6, 7]);

    let var = &mut file.add_variable::<u8>("two_unlim", &["x3", "x4"]).unwrap();
    var.set_fill_value(0u8).unwrap();
    let e = var.put_values(&[0u8, 1, 2, 3], None, None);
    assert_eq!(e.unwrap_err(), netcdf::error::Error::Ambiguous);
    var.put_values(&[0u8, 1, 2, 3], None, Some(&[1, 4]))
        .unwrap();
    let mut v = vec![0; 4];
    var.values_to(&mut v, None, Some(&[1, 4])).unwrap();
    assert_eq!(v, &[0u8, 1, 2, 3]);
    var.put_values(&[4u8, 5, 6], None, Some(&[3, 1])).unwrap();

    let mut v = vec![0; 4 * 3];
    var.values_to(&mut v, None, Some(&[3, 4])).unwrap();

    assert_eq!(v, &[4, 1, 2, 3, 5, 0, 0, 0, 6, 0, 0, 0]);
}

#[test]
fn length_of_variable() {
    let d = tempfile::tempdir().unwrap();
    let mut file = netcdf::create(d.path().join("variable_length.nc")).unwrap();

    file.add_dimension("x", 4).unwrap();
    file.add_dimension("y", 6).unwrap();
    file.add_unlimited_dimension("z").unwrap();

    let var = &mut file.add_variable::<f32>("x", &["x", "y"]).unwrap();
    assert_eq!(var.len(), 4 * 6);

    let var = &mut file.add_variable::<f64>("z", &["x", "z"]).unwrap();
    var.put_value(1u8, Some(&[2, 8])).unwrap();
    assert_eq!(var.len(), 4 * 9);
}

#[test]
fn single_length_variable() {
    let d = tempfile::tempdir().unwrap();
    let path = d.path().join("single_length_variable.nc");
    let mut file = netcdf::create(&path).unwrap();

    let var = &mut file.add_variable::<u8>("x", &[]).unwrap();

    var.put_value(3u8, None).unwrap();
    assert_eq!(var.value(Some(&[])), Ok(3u8));

    var.put_values::<u8>(&[], None, None).unwrap_err();
    assert_eq!(var.value(None), Ok(3u8));

    var.put_values::<u8>(&[2, 3], None, None).unwrap_err();

    var.put_values::<u8>(&[6], None, None).unwrap();
    assert_eq!(var.value(None), Ok(6u8));

    var.put_values::<u8>(&[8], Some(&[]), Some(&[])).unwrap();
    assert_eq!(var.value(None), Ok(8u8));

    var.put_values::<u8>(&[10], Some(&[1]), None).unwrap_err();
    assert_eq!(var.value(None), Ok(8u8));

    std::mem::drop(file);

    let file = netcdf::open(path).unwrap();

    let var = &file.variable("x").unwrap();

    assert_eq!(var.value::<u8>(None).unwrap(), 8);
}

#[test]
fn put_then_def() {
    let d = tempfile::tempdir().unwrap();
    let path = d.path().join("put_then_def.nc");
    let mut file = netcdf::create(path).unwrap();

    let var = &mut file.add_variable::<i8>("x", &[]).unwrap();
    var.put_value(3i8, None).unwrap();

    let var2 = &mut file.add_variable::<i8>("y", &[]).unwrap();
    var2.put_value(4i8, None).unwrap();
}

#[test]
fn string_variables() {
    let d = tempfile::tempdir().unwrap();
    let path = d.path().join("string_variables.nc");
    {
        let mut file = netcdf::create(&path).unwrap();

        file.add_unlimited_dimension("x").unwrap();
        file.add_dimension("y", 2).unwrap();

        let var = &mut file.add_string_variable("str", &["x"]).unwrap();

        var.put_string("Hello world!", None).unwrap();
        var.put_string(
            "Trying a very long string just to see how that goes",
            Some(&[2]),
        )
        .unwrap();
        var.put_string("Foreign letters: ßæøå, #41&i1/99", Some(&[3]))
            .unwrap();

        // Some weird interaction between unlimited dimensions, put_str,
        // and the name of this variable leads to crash. This
        // can be observed by changing this     \ /    to "x"
        let var = &mut file.add_variable::<i32>("y", &[]).unwrap();
        var.put_value(42i32, Some(&[])).unwrap();
    }
    let file = netcdf::open(path).unwrap();

    let var = &file.variable("str").unwrap();

    assert_eq!(var.string_value(Some(&[0])).unwrap(), "Hello world!");
    assert_eq!(var.string_value(Some(&[1])).unwrap(), "");
    assert_eq!(
        var.string_value(Some(&[2])).unwrap(),
        "Trying a very long string just to see how that goes"
    );
    assert_eq!(
        var.string_value(Some(&[3])).unwrap(),
        "Foreign letters: ßæøå, #41&i1/99"
    );

    let var = &file.variable("y").unwrap();
    var.string_value(None).unwrap_err();
}

#[test]
fn unlimited_in_parents() {
    let d = tempfile::tempdir().expect("Could not create tempdir");
    let path = d.path().join("unlimited_in_parents.nc");
    {
        let mut file = netcdf::create(&path).unwrap();

        file.add_dimension("x", 0).unwrap();
        file.add_dimension("y", 0).unwrap();
        file.add_dimension("z0", 5).unwrap();
        let g = &mut file.add_group("g").unwrap();
        g.add_dimension("z1", 0).unwrap();
    }
    let mut file = netcdf::append(&path).unwrap();

    let g = &mut file.group_mut("g").unwrap();
    g.add_variable::<i16>("w", &["z1"]).unwrap();
    g.add_variable::<u16>("v", &["x"]).unwrap();
}

#[test]
fn dimension_identifiers() {
    let d = tempfile::tempdir().expect("Could not create tempdir");
    let path = d.path().join("dimension_identifiers.nc");
    {
        let mut file = netcdf::create(&path).unwrap();

        // Create groups and dimensions
        let dim = &file.add_dimension("x", 10).unwrap();
        let vrootid = dim.identifier();
        let g = &mut file.add_group("g").unwrap();
        let dim = &g.add_dimension("x", 5).unwrap();
        let vgid = dim.identifier();
        let gg = file.group_mut("g").unwrap().add_group("g").unwrap();
        let dim = &gg.add_dimension("x", 7).unwrap();
        let vggid = dim.identifier();

        // Create variables
        file.add_variable_from_identifiers::<i8>("v_self_id", &[vrootid])
            .unwrap();
        let g = &mut file.group_mut("g").unwrap();
        g.add_variable_from_identifiers::<i8>("v_root_id", &[vrootid])
            .unwrap();
        g.add_variable_from_identifiers::<i8>("v_self_id", &[vgid])
            .unwrap();

        let gg = &mut g.group_mut("g").unwrap();
        gg.add_variable_from_identifiers::<i8>("v_root_id", &[vrootid])
            .unwrap();
        gg.add_variable_from_identifiers::<i8>("v_up_id", &[vgid])
            .unwrap();
        gg.add_variable_from_identifiers::<i8>("v_self_id", &[vggid])
            .unwrap();
    }

    let file = &netcdf::open(path).unwrap();

    assert_eq!(file.variable("v_self_id").unwrap().len(), 10);
    assert_eq!(
        file.group("g")
            .unwrap()
            .variable("v_root_id")
            .unwrap()
            .len(),
        10
    );
    assert_eq!(
        file.group("g")
            .unwrap()
            .variable("v_self_id")
            .unwrap()
            .len(),
        5
    );
    assert_eq!(
        file.group("g")
            .unwrap()
            .group("g")
            .unwrap()
            .variable("v_self_id")
            .unwrap()
            .len(),
        7
    );
    assert_eq!(
        file.group("g")
            .unwrap()
            .group("g")
            .unwrap()
            .variable("v_up_id")
            .unwrap()
            .len(),
        5
    );
    assert_eq!(
        file.group("g")
            .unwrap()
            .group("g")
            .unwrap()
            .variable("v_root_id")
            .unwrap()
            .len(),
        10
    );
}

#[test]
/// Test setting/getting endian value when creating a Variable
fn set_get_endian() {
	use netcdf::variable::Endianness;
    let d = tempfile::tempdir().unwrap();
    let f = d.path().join("append.nc");
    let dim_name = "some_dimension";
    for i in &[Endianness::Little, Endianness::Big]
    {
		{
			// first creates a simple netCDF file
			// and create a variable called "some_variable" in it
			let mut file_w = netcdf::File::create(&f).unwrap();
			file_w.root_mut().add_dimension(dim_name, 3).unwrap();
			let var = &mut file_w
				.root_mut()
				.add_variable::<i32>("some_variable", &[dim_name])
				.unwrap();
			var.endian(*i).unwrap();
			assert_eq!(var.endian_value(), Ok(*i));
			var.put_values::<i32>(&[1, 2, 3], None, None).unwrap();
			// close it (done when `file_w` goes out of scope)
		}
		{
			// re-open it
			// and get "some variable" endian_value
			let mut file_o = netcdf::open(&f).unwrap();
			let var = &mut file_o
				.root_mut()
				.variable("some_variable")
				.unwrap();
			assert_eq!(var.endian_value(), Ok(*i));
			// close it (done when `file_a` goes out of scope)
		}
	}
}

