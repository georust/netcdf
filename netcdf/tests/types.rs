mod common;

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
            }
            "u8" => {
                assert!(var.vartype().as_basic().unwrap().is_u8());
                assert!(var.vartype().is_u8());
            }
            "i16" => {
                assert!(var.vartype().as_basic().unwrap().is_i16());
                assert!(var.vartype().is_i16());
            }
            "u16" => {
                assert!(var.vartype().as_basic().unwrap().is_u16());
                assert!(var.vartype().is_u16());
            }
            "i32" => {
                assert!(var.vartype().as_basic().unwrap().is_i32());
                assert!(var.vartype().is_i32());
            }
            "u32" => {
                assert!(var.vartype().as_basic().unwrap().is_u32());
                assert!(var.vartype().is_u32());
            }
            "i64" => {
                assert!(var.vartype().as_basic().unwrap().is_i64());
                assert!(var.vartype().is_i64());
            }
            "u64" => {
                assert!(var.vartype().as_basic().unwrap().is_u64());
                assert!(var.vartype().is_u64());
            }
            "f32" => {
                assert!(var.vartype().as_basic().unwrap().is_f32());
                assert!(var.vartype().is_f32());
            }
            "f64" => {
                assert!(var.vartype().as_basic().unwrap().is_f64());
                assert!(var.vartype().is_f64());
            }
            "string" => assert!(var.vartype().is_string()),
            _ => panic!("Got an unexpected varname: {}", var.name()),
        }
    }
}

#[test]
fn add_opaque() {
    let d = tempfile::tempdir().unwrap();
    let path = d.path().join("test_opaque.nc");

    {
        let mut file = netcdf::create(path).unwrap();

        let typ = file.add_opaque_type("opa", 42).unwrap();
        assert_eq!(&typ.name(), "opa");
        assert_eq!(typ.size(), 42);

        let mut g = file.add_group("g").unwrap();
        let gtyp = g.add_opaque_type("oma", 43).unwrap();
        assert_eq!(&gtyp.name(), "oma");
        assert_eq!(gtyp.size(), 43);
    }

    // let file = netcdf::open(&path).unwrap();
    // let var = file.typ("opa").unwrap();
}

#[test]
fn add_vlen() {
    let d = tempfile::tempdir().unwrap();
    let path = d.path().join("test_add_vlen.nc");

    {
        let mut file = netcdf::create(path).unwrap();

        let typ = file.add_vlen_type::<u32>("v").unwrap();
        assert_eq!(&typ.name(), "v");
        assert!(typ.typ().is_u32());
        let mut g = file.add_group("g").unwrap();
        let typ = g.add_vlen_type::<i32>("w").unwrap();
        assert_eq!(&typ.name(), "w");
        assert!(&typ.typ().is_i32());
    }
}

#[test]
fn add_enum() {
    let d = tempfile::tempdir().unwrap();
    let path = d.path().join("test_add_enum.nc");

    {
        let mut file = netcdf::create(&path).unwrap();

        let e = file
            .add_enum_type::<i32>("e", &[("a", 0), ("b", 1), ("c", 2), ("d", 3)])
            .unwrap();
        assert_eq!(&e.name(), "e");
        assert!(e.typ().is_i32());
        for member in e.members::<i32>().unwrap() {
            match member.0.as_str() {
                "a" => assert_eq!(member.1, 0),
                "b" => assert_eq!(member.1, 1),
                "c" => assert_eq!(member.1, 2),
                "d" => assert_eq!(member.1, 3),
                _ => panic!(),
            }
        }
        assert_eq!(&e.name_from_value(0).unwrap(), "a");
        assert_eq!(&e.name_from_value(1).unwrap(), "b");
        assert_eq!(&e.name_from_value(2).unwrap(), "c");
        assert_eq!(&e.name_from_value(3).unwrap(), "d");
        assert!(&e.name_from_value(4).is_none());

        let mut g = file.add_group("g").unwrap();
        let e = g
            .add_enum_type::<i64>("e", &[("e", -32), ("f", 41), ("g", 1241232), ("h", 0)])
            .unwrap();
        assert_eq!(&e.name(), "e");
        assert!(e.typ().is_i64());
        for member in e.members::<i64>().unwrap() {
            match member.0.as_str() {
                "e" => assert_eq!(member.1, -32),
                "f" => assert_eq!(member.1, 41),
                "g" => assert_eq!(member.1, 1241232),
                "h" => assert_eq!(member.1, 0),
                _ => panic!(),
            }
        }
        assert_eq!(&e.name_from_value(-32).unwrap(), "e");
        assert_eq!(&e.name_from_value(41).unwrap(), "f");
        assert_eq!(&e.name_from_value(1241232).unwrap(), "g");
        assert_eq!(&e.name_from_value(0).unwrap(), "h");
        assert!(&e.name_from_value(4).is_none());
    }
}

#[test]
fn add_compound() {
    let d = tempfile::tempdir().unwrap();
    let path = d.path().join("test_add_compound.nc");
    let mut file = netcdf::create(path).unwrap();

    let mut builder = file.add_compound_type("c").unwrap();
    builder.add::<u8>("u8").unwrap();
    builder.add::<i8>("i8").unwrap();
    builder.add_array::<i32>("ai32", &[1, 2, 3]).unwrap();

    let c = builder.build().unwrap();
    let e = file.add_enum_type("e", &[("a", 1), ("b", 2)]).unwrap();

    let mut builder = file.add_compound_type("cc").unwrap();
    builder.add_type("e", &e.into()).unwrap();
    builder.add_type("c", &c.into()).unwrap();
    builder.build().unwrap();
}

#[test]
fn read_compound_simple_nc4() {
    use netcdf::types::VariableType;
    let path = common::test_location().join("simple_nc4.nc");
    let file = netcdf::open(&path).unwrap();

    let group = file.group("grp2").unwrap().unwrap();
    for typ in group.types() {
        let c = match typ {
            VariableType::Compound(c) => c,
            _ => panic!(),
        };
        assert_eq!(&c.name(), "sample_compound_type");
        let subtypes = c.fields().collect::<Vec<_>>();
        assert_eq!(subtypes.len(), 2);

        assert_eq!(&subtypes[0].name(), "i1");
        assert_eq!(&subtypes[1].name(), "i2");
        assert!(subtypes[0].typ().is_i32());
        assert!(subtypes[1].typ().is_i32());

        assert_eq!(subtypes[0].offset(), 0);
        assert_eq!(subtypes[1].offset(), std::mem::size_of::<i32>());

        assert!(subtypes[0].dimensions().is_none());
        assert!(subtypes[1].dimensions().is_none());
    }

    let var = group.variable("data").unwrap();

    if let VariableType::Compound(_) = var.vartype() {
    } else {
        panic!();
    }

    let mut raws = vec![0_u8; 12 * 6 * 2 * 4];
    var.raw_values(&mut raws, (..6, ..12)).unwrap();

    use std::convert::TryInto;
    let intlen = 4;
    for i in 0..6 * 12 {
        let i1 = i32::from_le_bytes(raws[2 * intlen * i..][..intlen].try_into().unwrap());
        let i2 = i32::from_le_bytes(
            raws[2 * intlen * i + intlen..][..intlen]
                .try_into()
                .unwrap(),
        );
        assert_eq!(i1, 42);
        assert_eq!(i2, -42);
    }
}

#[test]
fn put_get_enum() {
    let d = tempfile::tempdir().unwrap();
    let path = d.path().join("test_put_get_enum.nc");

    let bytes = (0..2 * 5).map(|i| i % 3 + 1).collect::<Vec<_>>();

    {
        let mut file = netcdf::create(&path).unwrap();
        let e = file
            .add_enum_type("e", &[("one", 1_u8), ("two", 2), ("three", 3)])
            .unwrap();
        file.add_dimension("x", 2).unwrap();
        file.add_dimension("y", 5).unwrap();

        let mut var = file
            .add_variable_with_type("var", &["y", "x"], &e.into())
            .unwrap();

        unsafe {
            var.put_raw_values(&bytes, (..5, ..2)).unwrap();
        }
    }

    let file = netcdf::open(&path).unwrap();
    let var = file.variable("var").unwrap();

    let mut bytes_copy = vec![0_u8; 5 * 2];
    var.raw_values(&mut bytes_copy, (..5, ..2)).unwrap();
    assert_eq!(bytes, bytes_copy);
}

#[test]
fn put_get_vlen() {
    let d = tempfile::tempdir().unwrap();
    let path = d.path().join("test_put_get_enum.nc");

    {
        let mut file = netcdf::create(&path).unwrap();
        file.add_dimension("x", 9).unwrap();
        let v = file.add_vlen_type::<i32>("v").unwrap();

        let mut var = file
            .add_variable_with_type("var", &["x"], &v.into())
            .unwrap();

        let buf = (0..9).collect::<Vec<i32>>();

        for i in 0..9 {
            var.put_vlen(&buf[i..], [i]).unwrap();
        }
    }

    let file = netcdf::open(&path).unwrap();
    let var = file.variable("var").unwrap();

    let buf = (0..9).collect::<Vec<i32>>();
    for i in 0..9 {
        let v = var.vlen::<i32, _>(&[i]).unwrap();
        assert_eq!(v, &buf[i..]);
    }
}

#[test]
fn char() {
    use netcdf::types::BasicType;
    let d = tempfile::tempdir().unwrap();
    let path = d.path().join("test_char.nc");

    let mut f = netcdf::create(path).unwrap();

    f.add_dimension("x", 2).unwrap();

    let mut var = f
        .add_variable_with_type("x", &["x"], &BasicType::Char.into())
        .unwrap();

    let vals = [2, 3];
    unsafe {
        var.put_raw_values(&vals, [..vals.len()]).unwrap();
    }

    let mut retrieved_vals = [0, 0];
    var.raw_values(&mut retrieved_vals, 0..2).unwrap();
    assert_eq!(vals, retrieved_vals);
}
