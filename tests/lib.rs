extern crate netcdf;

extern crate ndarray;
use ndarray::ArrayD;
use netcdf::{test_file, test_file_new};

// Failure tests
#[test]
#[should_panic(expected = "No such file or directory")]
fn bad_filename() {
    let f = test_file("blah_stuff.nc");
    let _file = netcdf::open(&f).unwrap();
}

// Read tests
#[test]
fn root_dims() {
    let f = test_file("simple_xy.nc");

    let file = netcdf::open(&f).unwrap();
    assert_eq!(f, file.name);

    assert_eq!(file.root.dimensions.get("x").unwrap().len, 6);
    assert_eq!(file.root.dimensions.get("y").unwrap().len, 12);
}

#[test]
fn global_attrs() {
    let f = test_file("patmosx_v05r03-preliminary_NOAA-19_asc_d20130630_c20140325.nc");

    let file = netcdf::open(&f).unwrap();
    assert_eq!(f, file.name);

    let ch1_attr = file.root.attributes.get("CH1_DARK_COUNT").unwrap();
    let ch1 = ch1_attr.get_float(false).unwrap();
    let eps = 1e-6;
    assert!((ch1-40.65863).abs() < eps);
    let ch1 = ch1_attr.get_int(true).unwrap();
    assert_eq!(ch1, 40);

    let sensor_attr = file.root.attributes.get("sensor").unwrap();
    let sensor_data = sensor_attr.get_char(false).unwrap();
    assert_eq!("AVHRR/3".to_string(), sensor_data);
}

#[test]
fn var_cast() {
    let f = test_file("simple_xy.nc");

    let file = netcdf::open(&f).unwrap();
    assert_eq!(f, file.name);

    let var = file.root.variables.get("data").unwrap();
    let data : Vec<i32> = var.get_int(false).unwrap();

    assert_eq!(data.len(), 6*12);
    for x in 0..(6*12) {
        assert_eq!(data[x], x as i32);
    }

    // do the same thing but cast to float
    let data : Vec<f32> = var.get_float(true).unwrap();

    assert_eq!(data.len(), 6*12);
    for x in 0..(6*12) {
        assert_eq!(data[x], x as f32);
    }
}

#[test]
fn test_index_fetch() {
    let f = test_file("simple_xy.nc");

    let file = netcdf::open(&f).unwrap();

    let var = file.root.variables.get("data").unwrap();
    let first_val: i32 = var.value_at(&[0usize, 0usize]).unwrap();
    let other_val: i32 = var.value_at(&[5, 3]).unwrap();

    assert_eq!(first_val, 0 as i32);
    assert_eq!(other_val, 63 as i32 );
}

#[test]
/// Tests implicit casts
fn implicit_cast() {
    let f = test_file("simple_xy.nc");

    let file = netcdf::open(&f).unwrap();
    assert_eq!(f, file.name);

    let var = file.root.variables.get("data").unwrap();
    let data : Vec<i32> = var.values().unwrap();

    assert_eq!(data.len(), 6*12);
    for x in 0..(6*12) {
        assert_eq!(data[x], x as i32);
    }

    // do the same thing but cast to float
    let data : Vec<f32> = var.values().unwrap();

    assert_eq!(data.len(), 6*12);
    for x in 0..(6*12) {
        assert_eq!(data[x], x as f32);
    }
}

#[test]
#[should_panic(expected = "Types are not equivalent and cast==false")]
fn var_cast_fail() {
    let f = test_file("simple_xy.nc");

    let file = netcdf::open(&f).unwrap();
    let var = file.root.variables.get("data").unwrap();

    // getting int Variable as float with false argument should fail.
    let _data : Vec<f32> = var.get_float(false).unwrap();
}

#[test]
fn last_dim_varies_fastest() {
    let f = test_file("simple_xy.nc");

    let file = netcdf::open(&f).unwrap();
    assert_eq!(f, file.name);

    let var = file.root.variables.get("data").unwrap();
    let data : Vec<i32> = var.get_int(false).unwrap();

    let nx = var.dimensions[0].len;
    let ny = var.dimensions[1].len;

    assert_eq!(nx, 6);
    assert_eq!(ny, 12);
    assert_eq!(nx*ny, var.len);

    for x in 0..nx {
        for y in 0..ny {
            let ind = x*nx + y;
            assert_eq!(data[ind as usize], ind as i32);
        }
    }
}

#[test]
fn open_pres_temp_4d() {
    let f = test_file("pres_temp_4D.nc");

    let file = netcdf::open(&f).unwrap();
    assert_eq!(f, file.name);

    let pres = file.root.variables.get("pressure").unwrap();
    assert_eq!(pres.dimensions[0].name, "time");
    assert_eq!(pres.dimensions[1].name, "level");
    assert_eq!(pres.dimensions[2].name, "latitude");
    assert_eq!(pres.dimensions[3].name, "longitude");

    // test var attributes
    assert_eq!(pres.attributes.get("units").unwrap().get_char(false).unwrap(), 
               "hPa".to_string());
}

#[test]
fn nc4_groups() {
    let f = test_file("simple_nc4.nc");

    let file = netcdf::open(&f).unwrap();
    assert_eq!(f, file.name);

    let grp1 = file.root.sub_groups.get("grp1").unwrap();
    assert_eq!(grp1.name, "grp1".to_string());

    let var = grp1.variables.get("data").unwrap();
    let data : Vec<i32> = var.get_int(true).unwrap();
    for x in 0..(6*12) {
        assert_eq!(data[x], x as i32);
    }
}

// Write tests
#[test]
fn create() {
    let f = test_file_new("create.nc");

    let file = netcdf::create(&f).unwrap();
    assert_eq!(f, file.name);
}

#[test]
fn def_dims_vars_attrs() {
    {
        let f = test_file_new("def_dims_vars_attrs.nc");

        let mut file = netcdf::create(&f).unwrap();

        let dim1_name = "ljkdsjkldfs";
        let dim2_name = "dsfkdfskl";
        file.root.add_dimension(dim1_name, 10).unwrap();
        file.root.add_dimension(dim2_name, 20).unwrap();
        assert_eq!(file.root.dimensions.get(dim1_name).unwrap().len, 10);
        assert_eq!(file.root.dimensions.get(dim2_name).unwrap().len, 20);

        let var_name = "varstuff_int";
        let data : Vec<i32> = vec![42; (10*20)];
        file.root.add_variable(
                    var_name, 
                    &vec![dim1_name.to_string(), dim2_name.to_string()],
                    &data
                ).unwrap();
        assert_eq!(file.root.variables.get(var_name).unwrap().len, 20*10);

        let var_name = "varstuff_float";
        let data : Vec<f32> = vec![42.2; 10];
        file.root.add_variable(
                    var_name, 
                    &vec![dim1_name.to_string()],
                    &data
                ).unwrap();
        assert_eq!(file.root.variables.get(var_name).unwrap().len, 10);

        // test global attrs
        file.root.add_attribute(
                "testattr1",
                3,
            ).unwrap();
        file.root.add_attribute(
                "testattr2",
                "Global string attr".to_string(),
            ).unwrap();

        // test var attrs
        file.root.variables.get_mut(var_name).unwrap().add_attribute(
                "varattr1",
                5,
            ).unwrap();
        file.root.variables.get_mut(var_name).unwrap().add_attribute(
                "varattr2",
                "Variable string attr".to_string(),
            ).unwrap();
    }

    // now, read in the file we created and verify everything
    {
        let f = test_file_new("def_dims_vars_attrs.nc");

        let file = netcdf::open(&f).unwrap();

        // verify dimensions
        let dim1_name = "ljkdsjkldfs";
        let dim2_name = "dsfkdfskl";
        let dim1 = file.root.dimensions.get(dim1_name).unwrap();
        let dim2 = file.root.dimensions.get(dim2_name).unwrap();
        assert_eq!(dim1.len, 10);
        assert_eq!(dim2.len, 20);

        // verify variable data
        let var_name = "varstuff_int";
        let data_test : Vec<i32> = vec![42; (10*20)];
        let data_file : Vec<i32> = 
            file.root.variables.get(var_name).unwrap().get_int(false).unwrap();
        assert_eq!(data_test.len(), data_file.len());
        for i in 0..data_test.len() {
            assert_eq!(data_test[i], data_file[i]);
        }

        let var_name = "varstuff_float";
        let data_test : Vec<f32> = vec![42.2; 10];
        let data_file : Vec<f32> = 
            file.root.variables.get(var_name).unwrap().get_float(false).unwrap();
        assert_eq!(data_test.len(), data_file.len());
        for i in 0..data_test.len() {
            assert_eq!(data_test[i], data_file[i]);
        }
        
        // verify global attrs
        assert_eq!(3, 
          file.root.attributes.get("testattr1").unwrap().get_int(false).unwrap());
        assert_eq!("Global string attr".to_string(), 
          file.root.attributes.get("testattr2").unwrap().get_char(false).unwrap());
        
        // verify var attrs
        assert_eq!(5,
          file.root.variables.get(var_name).unwrap()
            .attributes.get("varattr1").unwrap().get_int(false).unwrap());
        assert_eq!("Variable string attr",
          file.root.variables.get(var_name).unwrap()
            .attributes.get("varattr2").unwrap().get_char(false).unwrap());

    }
}

#[test]
fn all_var_types() {
    // write
    {
        let f = test_file_new("all_var_types.nc");
        let mut file = netcdf::create(&f).unwrap();

        let dim_name = "dim1";
        file.root.add_dimension(dim_name, 10).unwrap();

        // byte
        let data : Vec<i8> = vec![42 as i8; 10];
        let var_name = "var_byte";
        file.root.add_variable(
                    var_name, 
                    &vec![dim_name.to_string()],
                    &data
                ).unwrap();
        // short
        let data : Vec<i16> = vec![42 as i16; 10];
        let var_name = "var_short";
        file.root.add_variable(
                    var_name, 
                    &vec![dim_name.to_string()],
                    &data
                ).unwrap();
        // ushort
        let data : Vec<u16> = vec![42 as u16; 10];
        let var_name = "var_ushort";
        file.root.add_variable(
                    var_name, 
                    &vec![dim_name.to_string()],
                    &data
                ).unwrap();
        // int
        let data : Vec<i32> = vec![42 as i32; 10];
        let var_name = "var_int";
        file.root.add_variable(
                    var_name, 
                    &vec![dim_name.to_string()],
                    &data
                ).unwrap();
        // uint
        let data : Vec<u32> = vec![42 as u32; 10];
        let var_name = "var_uint";
        file.root.add_variable(
                    var_name, 
                    &vec![dim_name.to_string()],
                    &data
                ).unwrap();
        // int64
        let data : Vec<i64> = vec![42 as i64; 10];
        let var_name = "var_int64";
        file.root.add_variable(
                    var_name, 
                    &vec![dim_name.to_string()],
                    &data
                ).unwrap();
        // uint64
        let data : Vec<u64> = vec![42 as u64; 10];
        let var_name = "var_uint64";
        file.root.add_variable(
                    var_name, 
                    &vec![dim_name.to_string()],
                    &data
                ).unwrap();
        // float
        let data : Vec<f32> = vec![42.2 as f32; 10];
        let var_name = "var_float";
        file.root.add_variable(
                    var_name, 
                    &vec![dim_name.to_string()],
                    &data
                ).unwrap();
        // double
        let data : Vec<f64> = vec![42.2 as f64; 10];
        let var_name = "var_double";
        file.root.add_variable(
                    var_name, 
                    &vec![dim_name.to_string()],
                    &data
                ).unwrap();
    }

    // read
    {
        let f = test_file_new("all_var_types.nc");
        let file = netcdf::open(&f).unwrap();

        // byte
        let data : Vec<i8> = 
            file.root.variables.get("var_byte").unwrap().get_byte(false).unwrap();
        for i in 0..10 {
            assert_eq!(42 as i8, data[i]);
        }
        // short
        let data : Vec<i16> = 
            file.root.variables.get("var_short").unwrap().get_short(false).unwrap();
        for i in 0..10 {
            assert_eq!(42 as i16, data[i]);
        }
        // ushort
        let data : Vec<u16> = 
            file.root.variables.get("var_ushort").unwrap().get_ushort(false).unwrap();
        for i in 0..10 {
            assert_eq!(42 as u16, data[i]);
        }
        // int
        let data : Vec<i32> = 
            file.root.variables.get("var_int").unwrap().get_int(false).unwrap();
        for i in 0..10 {
            assert_eq!(42 as i32, data[i]);
        }
        // uint
        let data : Vec<u32> = 
            file.root.variables.get("var_uint").unwrap().get_uint(false).unwrap();
        for i in 0..10 {
            assert_eq!(42 as u32, data[i]);
        }
        // int64
        let data : Vec<i64> = 
            file.root.variables.get("var_int64").unwrap().get_int64(false).unwrap();
        for i in 0..10 {
            assert_eq!(42 as i64, data[i]);
        }
        // uint64
        let data : Vec<u64> = 
            file.root.variables.get("var_uint64").unwrap().get_uint64(false).unwrap();
        for i in 0..10 {
            assert_eq!(42 as u64, data[i]);
        }
        // float
        let data : Vec<f32> = 
            file.root.variables.get("var_float").unwrap().get_float(false).unwrap();
        for i in 0..10 {
            assert_eq!(42.2 as f32, data[i]);
        }
        // double
        let data : Vec<f64> = 
            file.root.variables.get("var_double").unwrap().get_double(false).unwrap();
        for i in 0..10 {
            assert_eq!(42.2 as f64, data[i]);
        }
        
    }

}

#[test]
fn all_attr_types() {
    {
        let f = test_file_new("all_attr_types.nc");
        let mut file = netcdf::create(&f).unwrap();

        // byte
        file.root.add_attribute(
                "attr_byte",
                3 as i8,
            ).unwrap();
        // short
        file.root.add_attribute(
                "attr_short",
                3 as i16,
            ).unwrap();
        // ushort
        file.root.add_attribute(
                "attr_ushort",
                3 as u16,
            ).unwrap();
        // int
        file.root.add_attribute(
                "attr_int",
                3 as i32,
            ).unwrap();
        // uint
        file.root.add_attribute(
                "attr_uint",
                3 as u32,
            ).unwrap();
        // int64
        file.root.add_attribute(
                "attr_int64",
                3 as i64,
            ).unwrap();
        // uint64
        file.root.add_attribute(
                "attr_uint64",
                3 as u64,
            ).unwrap();
        // float
        file.root.add_attribute(
                "attr_float",
                3.2 as f32,
            ).unwrap();
        // double
        file.root.add_attribute(
                "attr_double",
                3.2 as f64,
            ).unwrap();
    }

    {
        let f = test_file_new("all_attr_types.nc");
        let file = netcdf::open(&f).unwrap();

        // byte
        assert_eq!(3 as i8, 
          file.root.attributes.get("attr_byte").unwrap().get_byte(false).unwrap());
        // short
        assert_eq!(3 as i16, 
          file.root.attributes.get("attr_short").unwrap().get_short(false).unwrap());
        // ushort
        assert_eq!(3 as u16, 
          file.root.attributes.get("attr_ushort").unwrap().get_ushort(false).unwrap());
        // int
        assert_eq!(3 as i32, 
          file.root.attributes.get("attr_int").unwrap().get_int(false).unwrap());
        // uint
        assert_eq!(3 as u32, 
          file.root.attributes.get("attr_uint").unwrap().get_uint(false).unwrap());
        // int64
        assert_eq!(3 as i64, 
          file.root.attributes.get("attr_int64").unwrap().get_int64(false).unwrap());
        // uint64
        assert_eq!(3 as u64, 
          file.root.attributes.get("attr_uint64").unwrap().get_uint64(false).unwrap());
        // float
        assert_eq!(3.2 as f32, 
          file.root.attributes.get("attr_float").unwrap().get_float(false).unwrap());
        // double
        assert_eq!(3.2 as f64, 
          file.root.attributes.get("attr_double").unwrap().get_double(false).unwrap());

    }
}

#[test]
/// Tests the shape of a variable
/// when fetched using "Variable::as_array()"
fn fetch_ndarray() {
    let f = test_file("pres_temp_4D.nc");
    let file = netcdf::open(&f).unwrap();
    assert_eq!(f, file.name);
    let pres = file.root.variables.get("pressure").unwrap();
    let values_array: ArrayD<f64>  = pres.as_array().unwrap();
    assert_eq!(values_array.shape(),  &[2, 2, 6, 12]);
}

#[test]
// assert slice fetching
fn fetch_slice() {
    let f = test_file("simple_xy.nc");
    let file = netcdf::open(&f).unwrap();
    assert_eq!(f, file.name);
    let pres = file.root.variables.get("data").unwrap();
    let values: Vec<i32>  = pres.values_at(&[0, 0], &[6, 3]).unwrap();
    let expected_values: [i32; 18] = [
        0,  1,  2, 12, 13, 14, 24, 25, 26, 36, 37, 38, 48, 49, 50, 60, 61, 62];
    for i in 0..values.len() {
        assert_eq!(expected_values[i], values[i]);
    }
}

#[test]
// assert slice fetching
fn fetch_slice_as_ndarray() {
    let f = test_file("simple_xy.nc");
    let file = netcdf::open(&f).unwrap();
    assert_eq!(f, file.name);
    let pres = file.root.variables.get("data").unwrap();
    let values_array: ArrayD<i32> = pres.array_at(&[0, 0], &[6, 3]).unwrap();
    assert_eq!(values_array.shape(), &[6, 3]);
}

#[test]
// test file modification
fn append() {
    let f = test_file_new("append.nc");
    let dim_name = "some_dimension";
    {
        // first creates a simple netCDF file
        // and create a variable called "some_variable" in it
        let mut file_w = netcdf::create(&f).unwrap();
        file_w.root.add_dimension(dim_name, 3).unwrap();
        file_w.root.add_variable(
                    "some_variable", 
                    &vec![dim_name.into()],
                    &vec![1., 2., 3.]
                ).unwrap();
        // close it (done when `file_w` goes out of scope)
    }
    {
        // re-open it in append mode
        // and create a variable called "some_other_variable"
        let mut file_a = netcdf::append(&f).unwrap();
        file_a.root.add_variable(
                    "some_other_variable", 
                    &vec![dim_name.into()],
                    &vec![2., 4., 6.]
                ).unwrap();
        // close it (done when `file_a` goes out of scope)
    }
    // finally open  the file in read only mode
    // and test the existence of both variable 
    let file = netcdf::append(&f).unwrap();
    assert!(file.root.variables.contains_key("some_variable"));
    assert!(file.root.variables.contains_key("some_other_variable"));
}

#[test]
// test file modification
fn put_single_value() {
    let f = test_file_new("append_value.nc");
    let dim_name = "some_dimension";
    let var_name = "some_variable";
    {
        // first creates a simple netCDF file
        // and create a variable called "some_variable" in it
        let mut file_w = netcdf::create(&f).unwrap();
        file_w.root.add_dimension(dim_name, 3).unwrap();
        file_w.root.add_variable(
                    var_name,
                    &vec![dim_name.into()],
                    &vec![1., 2., 3.]
                ).unwrap();
        // close it (done when `file_w` goes out of scope)
    }
    let indices: [usize; 1] = [0];
    {
        // re-open it in append mode
        let mut file_a = netcdf::append(&f).unwrap();
        let mut var = file_a.root.variables.get_mut(var_name).unwrap();
        let res = var.put_value_at(100., &indices);
        assert_eq!(res, Ok(()));
        // close it (done when `file_a` goes out of scope)
    }
    // finally open  the file in read only mode
    // and test the values of 'some_variable'
    let file = netcdf::open(&f).unwrap();
    let var = file.root.variables.get(var_name).unwrap();
    assert_eq!(var.value_at(&indices), Ok(100.));
}

#[test]
// test file modification
fn put_values() {
    let f = test_file_new("append_values.nc");
    let dim_name = "some_dimension";
    let var_name = "some_variable";
    {
        // first creates a simple netCDF file
        // and create a variable called "some_variable" in it
        let mut file_w = netcdf::create(&f).unwrap();
        file_w.root.add_dimension(dim_name, 3).unwrap();
        file_w.root.add_variable(
                    var_name,
                    &vec![dim_name.into()],
                    &vec![1., 2., 3.]
                ).unwrap();
        // close it (done when `file_w` goes out of scope)
    }
    let indices: [usize; 1] = [1];
    let values: [f32; 2] = [100., 200.];
    {
        // re-open it in append mode
        let mut file_a = netcdf::append(&f).unwrap();
        let mut var = file_a.root.variables.get_mut(var_name).unwrap();
        let res = var.put_values_at(&values, &indices, &[values.len()]);
        assert_eq!(res, Ok(()));
        // close it (done when `file_a` goes out of scope)
    }
    // finally open  the file in read only mode
    // and test the values of 'some_variable'
    let file = netcdf::open(&f).unwrap();
    let var = file.root.variables.get(var_name).unwrap();
    assert_eq!(
        var.values_at::<f32>(&indices, &[values.len()]).unwrap().as_slice(),
        values
    );
}

#[test]
/// Test setting a fill value when creating a Variable
fn set_fill_value() {
    let f = test_file_new("fill_value.nc");
    let dim_name = "some_dimension";
    let var_name = "some_variable";
    let fill_value = -2. as f32;

    let mut file_w = netcdf::create(&f).unwrap();
    file_w.root.add_dimension(dim_name, 3).unwrap();
    file_w.root.add_variable_with_fill_value(
        var_name,
        &vec![dim_name.into()],
        &vec![1. as f32, 2. as f32, 3. as f32],
        fill_value
    ).unwrap();
    let var =  file_w.root.variables.get(var_name).unwrap();
    let attr = var.attributes.get("_FillValue").unwrap().get_float(false).unwrap();
    // compare requested fill_value and attribute _FillValue
    assert_eq!(fill_value, attr);
}

#[test]
/// Test reading variable into a buffer
fn read_values_into_buffer() {
    let f = test_file("simple_xy.nc");
    let file = netcdf::open(&f).unwrap();
    let var = file.root.variables.get("data").unwrap();
    // pre-allocate the Array
    let mut data: Vec<i32> = Vec::with_capacity(var.len as usize);
    var.read_values_into_buffer(&mut data);

    assert_eq!(data.len(), 6*12);
    for x in 0..(6*12) {
        assert_eq!(data[x], x as i32);
    }
}

#[test]
/// Test reading a slice of a variable into a buffer
fn read_slice_into_buffer() {
    let f = test_file("simple_xy.nc");
    let file = netcdf::open(&f).unwrap();
    let pres = file.root.variables.get("data").unwrap();
    // pre-allocate the Array
    let mut values: Vec<i32>  = Vec::with_capacity(6 * 3);
    pres.read_slice_into_buffer(&[0, 0], &[6, 3], &mut values).unwrap();
    let expected_values: [i32; 18] = [
        0,  1,  2, 12, 13, 14, 24, 25, 26, 36, 37, 38, 48, 49, 50, 60, 61, 62];
    for i in 0..values.len() {
        assert_eq!(expected_values[i], values[i]);
    }
}
