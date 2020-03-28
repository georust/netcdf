#[test]
fn test_roundtrip_types() {
    let d = tempfile::tempdir().unwrap();
    let path = d.path().join("test_roundtrip_types.nc");
    {
        let mut file = netcdf::create(&path).unwrap();
        file.add_variable::<i8>("i8", &[]).unwrap();
        file.add_variable::<u8>("u8", &[]).unwrap();
        file.add_variable::<i16>("i16", &[]).unwrap();
        file.add_variable::<u16>("u16", &[]).unwrap();
        file.add_variable::<i32>("i32", &[]).unwrap();
        file.add_variable::<u32>("u32", &[]).unwrap();
        file.add_variable::<i64>("i64", &[]).unwrap();
        file.add_variable::<u64>("u64", &[]).unwrap();
        file.add_variable::<f32>("f32", &[]).unwrap();
        file.add_variable::<f64>("f64", &[]).unwrap();
        file.add_string_variable("string", &[]).unwrap();
    }

    let file = netcdf::open(&path).unwrap();
    assert_eq!(file.types().unwrap().count(), 0);
    let root = file.root().unwrap();
    assert_eq!(root.types().count(), 0);
    for var in file.variables() {
        match var.name().as_str() {
            "i8" => {
                assert!(var.vartype().as_basic().unwrap().is_i8());
                assert!(var.vartype().is_i8());
            },
            "u8" => {
                assert!(var.vartype().as_basic().unwrap().is_u8());
                assert!(var.vartype().is_u8());
            },
            "i16" => {
                assert!(var.vartype().as_basic().unwrap().is_i16());
                assert!(var.vartype().is_i16());
            },
            "u16" => {
                assert!(var.vartype().as_basic().unwrap().is_u16());
                assert!(var.vartype().is_u16());
            },
            "i32" => {
                assert!(var.vartype().as_basic().unwrap().is_i32());
                assert!(var.vartype().is_i32());
            },
            "u32" => {
                assert!(var.vartype().as_basic().unwrap().is_u32());
                assert!(var.vartype().is_u32());
            },
            "i64" => {
                assert!(var.vartype().as_basic().unwrap().is_i64());
                assert!(var.vartype().is_i64());
            },
            "u64" => {
                assert!(var.vartype().as_basic().unwrap().is_u64());
                assert!(var.vartype().is_u64());
            },
            "f32" => {
                assert!(var.vartype().as_basic().unwrap().is_f32());
                assert!(var.vartype().is_f32());
            },
            "f64" => {
                assert!(var.vartype().as_basic().unwrap().is_f64());
                assert!(var.vartype().is_f64());
            },
            "string" => assert!(var.vartype().is_string()),
            _ => panic!("Got an unexpected varname: {}", var.name()),
        }
    }
}
