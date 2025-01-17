use netcdf::types::*;

mod common;

#[test]
fn test_empty_basic_types() {
    let d = tempfile::tempdir().unwrap();
    let path = d.path().join("test_roundtrip_types.nc");
    #[repr(transparent)]
    struct NcString(*const std::ffi::c_char);

    unsafe impl NcTypeDescriptor for NcString {
        fn type_descriptor() -> NcVariableType {
            NcVariableType::String
        }
    }
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
        file.add_variable::<NcString>("string", &[]).unwrap();
    }

    let file = netcdf::open(&path).unwrap();
    assert_eq!(file.types().unwrap().count(), 0);

    let root = file.root().unwrap();
    assert_eq!(root.types().count(), 0);
    for var in file.variables() {
        match var.name().as_str() {
            "i8" => {
                assert_eq!(var.vartype(), NcVariableType::Int(IntType::I8));
            }
            "u8" => {
                assert_eq!(var.vartype(), NcVariableType::Int(IntType::U8));
            }
            "i16" => {
                assert_eq!(var.vartype(), NcVariableType::Int(IntType::I16));
            }
            "u16" => {
                assert_eq!(var.vartype(), NcVariableType::Int(IntType::U16));
            }
            "i32" => {
                assert_eq!(var.vartype(), NcVariableType::Int(IntType::I32));
            }
            "u32" => {
                assert_eq!(var.vartype(), NcVariableType::Int(IntType::U32));
            }
            "i64" => {
                assert_eq!(var.vartype(), NcVariableType::Int(IntType::I64));
            }
            "u64" => {
                assert_eq!(var.vartype(), NcVariableType::Int(IntType::U64));
            }
            "f32" => {
                assert_eq!(var.vartype(), NcVariableType::Float(FloatType::F32));
            }
            "f64" => {
                assert_eq!(var.vartype(), NcVariableType::Float(FloatType::F64));
            }
            "string" => assert_eq!(var.vartype(), NcVariableType::String),
            _ => panic!("Got an unexpected varname: {}", var.name()),
        }
    }
}

#[test]
fn add_opaque() {
    let d = tempfile::tempdir().unwrap();
    let path = d.path().join("test_opaque.nc");

    #[repr(transparent)]
    struct Foo([u8; 2]);

    unsafe impl NcTypeDescriptor for Foo {
        fn type_descriptor() -> NcVariableType {
            NcVariableType::Opaque(OpaqueType {
                name: "Foo".to_string(),
                size: std::mem::size_of::<Foo>(),
            })
        }
    }

    {
        let mut file = netcdf::create(&path).unwrap();
        file.add_type::<Foo>().unwrap();

        let mut g = file.add_group("g").unwrap();
        g.add_type::<Foo>().unwrap();
    }

    let file = netcdf::open(&path).unwrap();
    let types = file.types().unwrap().collect::<Vec<_>>();
    assert_eq!(types, &[Foo::type_descriptor()]);

    let group = file.group("g").unwrap().unwrap();
    let types = group.types().collect::<Vec<_>>();
    assert_eq!(types, &[Foo::type_descriptor()]);
}

#[test]
fn add_vlen() {
    let d = tempfile::tempdir().unwrap();
    let path = d.path().join("test_add_vlen.nc");

    {
        let mut file = netcdf::create(path).unwrap();

        let tp = NcVariableType::Vlen(VlenType {
            name: "v".to_owned(),
            basetype: Box::new(NcVariableType::Int(IntType::U32)),
        });

        file.add_type_from_descriptor(tp.clone()).unwrap();

        let types = file.types().unwrap().collect::<Vec<_>>();
        assert_eq!(types, &[tp]);

        let mut g = file.add_group("g").unwrap();
        let tp = NcVariableType::Vlen(VlenType {
            name: "w".to_owned(),
            basetype: Box::new(NcVariableType::Int(IntType::I32)),
        });
        g.add_type_from_descriptor(tp.clone()).unwrap();

        let types = g.types().collect::<Vec<_>>();
        assert_eq!(types, &[tp]);
    }
}

#[test]
#[cfg(feature = "derive")]
fn add_enum() {
    let d = tempfile::tempdir().unwrap();
    let path = d.path().join("test_add_enum.nc");

    {
        let mut file = netcdf::create(&path).unwrap();

        #[derive(netcdf_derive::NcType)]
        #[repr(i32)]
        #[allow(non_camel_case_types)]
        #[allow(dead_code)]
        enum e {
            a = 0,
            b,
            c,
            d,
        }

        file.add_type::<e>().unwrap();
        {
            let types = file.types().unwrap().collect::<Vec<_>>();
            assert_eq!(types.len(), 1);
            assert_eq!(
                types[0],
                NcVariableType::Enum(EnumType {
                    name: "e".to_owned(),
                    fieldnames: vec![
                        "a".to_owned(),
                        "b".to_owned(),
                        "c".to_owned(),
                        "d".to_owned()
                    ],
                    fieldvalues: EnumTypeValues::I32(vec![0, 1, 2, 3])
                })
            )
        }

        #[derive(netcdf_derive::NcType)]
        #[repr(i64)]
        #[allow(non_camel_case_types)]
        #[allow(dead_code)]
        enum f {
            e = -32,
            f = 41,
            g = 1241232,
            h = 0,
        }

        let mut g = file.add_group("g").unwrap();
        g.add_type::<f>().unwrap();
        {
            let types = g.types().collect::<Vec<_>>();
            assert_eq!(types.len(), 1);
            assert_eq!(
                types[0],
                NcVariableType::Enum(EnumType {
                    name: "f".to_owned(),
                    fieldnames: vec![
                        "e".to_owned(),
                        "f".to_owned(),
                        "g".to_owned(),
                        "h".to_owned()
                    ],
                    fieldvalues: EnumTypeValues::I64(vec![
                        f::e as _,
                        f::f as _,
                        f::g as _,
                        f::h as _
                    ])
                })
            )
        }
    }
}

#[test]
#[cfg(feature = "derive")]
fn add_compound() {
    let d = tempfile::tempdir().unwrap();
    let path = d.path().join("test_add_compound.nc");
    let mut file = netcdf::create(path).unwrap();

    #[derive(netcdf_derive::NcType)]
    #[repr(C)]
    #[netcdf(rename = "c")]
    struct C {
        u8: u8,
        i8: i8,
        i16: i16,
    }

    file.add_type::<C>().unwrap();

    #[derive(netcdf_derive::NcType)]
    #[repr(i32)]
    #[netcdf(rename = "e")]
    #[allow(dead_code)]
    enum E {
        #[netcdf(rename = "a")]
        A = 1,
        #[netcdf(rename = "b")]
        B = 2,
    }
    file.add_type::<E>().unwrap();

    #[derive(netcdf_derive::NcType)]
    #[repr(C)]
    #[netcdf(rename = "cc")]
    struct CC {
        e: E,
        c: C,
    }
    file.add_type::<CC>().unwrap();

    let types = file.types().unwrap().collect::<Vec<_>>();
    assert_eq!(
        types[0],
        NcVariableType::Compound(CompoundType {
            name: "c".to_owned(),
            size: std::mem::size_of::<C>(),
            fields: vec![
                CompoundTypeField {
                    name: "u8".to_owned(),
                    basetype: NcVariableType::Int(IntType::U8),
                    arraydims: None,
                    offset: 0,
                },
                CompoundTypeField {
                    name: "i8".to_owned(),
                    basetype: NcVariableType::Int(IntType::I8),
                    arraydims: None,
                    offset: 1,
                },
                CompoundTypeField {
                    name: "i16".to_owned(),
                    basetype: NcVariableType::Int(IntType::I16),
                    arraydims: None,
                    offset: 2,
                },
            ]
        })
    );
    assert_eq!(types[1], E::type_descriptor(),);
    assert_eq!(types[2], CC::type_descriptor());
}

#[test]
#[cfg(feature = "derive")]
fn read_compound_simple_nc4() {
    let path = common::test_location().join("simple_nc4.nc");
    let file = netcdf::open(&path).unwrap();

    #[repr(C)]
    #[derive(netcdf_derive::NcType, Copy, Clone, Debug, PartialEq, Eq)]
    #[netcdf(rename = "sample_compound_type")]
    struct SampleCompoundType {
        i1: i32,
        i2: i32,
    }

    let group = file.group("grp2").unwrap().unwrap();
    let types = group.types().collect::<Vec<_>>();
    assert_eq!(
        types,
        &[NcVariableType::Compound(CompoundType {
            name: "sample_compound_type".to_owned(),
            size: 8,
            fields: vec![
                CompoundTypeField {
                    name: "i1".to_owned(),
                    basetype: NcVariableType::Int(IntType::I32),
                    offset: 0,
                    arraydims: None,
                },
                CompoundTypeField {
                    name: "i2".to_owned(),
                    basetype: NcVariableType::Int(IntType::I32),
                    offset: 4,
                    arraydims: None,
                },
            ]
        })]
    );

    let var = group.variable("data").unwrap();

    assert_eq!(var.vartype(), SampleCompoundType::type_descriptor());

    let values = var.get_values::<SampleCompoundType, _>(..).unwrap();

    assert_eq!(
        values,
        [SampleCompoundType { i1: 42, i2: -42 }].repeat(6 * 12)
    );
}

#[test]
#[cfg(feature = "derive")]
fn put_get_enum() {
    let d = tempfile::tempdir().unwrap();
    let path = d.path().join("test_put_get_enum.nc");

    #[derive(netcdf::NcType, Debug, Copy, Clone, PartialEq, Eq)]
    #[repr(i8)]
    enum E {
        #[netcdf(rename = "one")]
        One = 1,
        #[netcdf(rename = "two")]
        Two = 2,
        #[netcdf(rename = "three")]
        Three = 3,
    }

    let bytes = [E::One, E::Two, E::Three].repeat(5);

    {
        let mut file = netcdf::create(&path).unwrap();
        file.add_type::<E>().unwrap();
        file.add_dimension("x", 4).unwrap();
        file.add_dimension("y", 5).unwrap();

        let mut var = file.add_variable::<E>("var", &["y", "x"]).unwrap();

        var.put_values(&bytes, (..5, ..3)).unwrap();
    }

    let file = netcdf::open(&path).unwrap();
    let var = file.variable("var").unwrap();

    let bytes_copy = var.get_values((..5, ..3)).unwrap();
    assert_eq!(bytes, bytes_copy);
}

#[test]
fn put_get_vlen() {
    let d = tempfile::tempdir().unwrap();
    let path = d.path().join("test_put_get_enum.nc");

    #[derive(Copy, Clone)]
    #[repr(C, packed)]
    struct Foo {
        len: usize,
        p: *mut u8,
    }

    unsafe impl NcTypeDescriptor for Foo {
        fn type_descriptor() -> NcVariableType {
            NcVariableType::Vlen(VlenType {
                name: "Foo".to_owned(),
                basetype: Box::new(NcVariableType::Int(IntType::U8)),
            })
        }
    }

    {
        let mut file = netcdf::create(&path).unwrap();
        file.add_dimension("x", 9).unwrap();
        file.add_type::<Foo>().unwrap();

        let mut var = file.add_variable::<Foo>("var", &["x"]).unwrap();

        let buf = (0..9).collect::<Vec<u8>>();
        let foo = Foo {
            len: buf.len(),
            p: buf.as_ptr().cast_mut(),
        };

        var.put_values(&[foo].repeat(9), ..).unwrap();
    }

    let file = netcdf::open(&path).unwrap();
    let var = file.variable("var").unwrap();

    let buf = (0..9).collect::<Vec<u8>>();
    let mut values = var.get_values::<Foo, _>(..).unwrap();
    for v in &values {
        assert_eq!(&buf, unsafe { std::slice::from_raw_parts(v.p, v.len) });
    }
    let errcode;
    unsafe {
        let _lock = netcdf_sys::libnetcdf_lock.lock();
        errcode = netcdf_sys::nc_free_vlens(values.len(), values.as_mut_ptr().cast());
    }
    assert_eq!(errcode, netcdf_sys::NC_NOERR);
}

#[test]
fn char() {
    let d = tempfile::tempdir().unwrap();
    let path = d.path().join("test_char.nc");

    #[repr(transparent)]
    #[derive(Copy, Clone, Debug, PartialEq, Eq)]
    struct NcChar(pub i8);

    unsafe impl NcTypeDescriptor for NcChar {
        fn type_descriptor() -> NcVariableType {
            NcVariableType::Char
        }
    }

    let mut f = netcdf::create(path).unwrap();
    f.add_dimension("x", 2).unwrap();

    let mut var = f.add_variable::<NcChar>("x", &["x"]).unwrap();

    assert_eq!(var.vartype(), NcVariableType::Char);

    let vals = [NcChar('2' as _), NcChar('3' as _)];
    var.put_values(&vals, [..vals.len()]).unwrap();

    let retrieved_vals = var.get_values::<NcChar, _>(0..2).unwrap();
    assert_eq!(vals.as_slice(), retrieved_vals);
}

#[test]
#[cfg(feature = "derive")]
fn no_subtype() {
    #[derive(netcdf::NcType)]
    #[repr(C)]
    struct Foo {
        a: u8,
    }
    #[derive(netcdf::NcType)]
    #[repr(C)]
    struct Bar {
        a: i8,
    }
    #[derive(netcdf::NcType)]
    #[repr(C)]
    struct FooBar {
        foo: Foo,
        bar: Bar,
    }
    let d = tempfile::tempdir().unwrap();
    let path = d.path().join("test_subtype.nc");

    let mut file = netcdf::create(path).unwrap();

    file.add_type::<FooBar>().unwrap_err();
    file.add_type::<Foo>().unwrap();
    file.add_type::<FooBar>().unwrap_err();
    file.add_type::<Bar>().unwrap();
    file.add_type::<FooBar>().unwrap();
}

#[test]
fn add_string_variable() {
    let d = tempfile::tempdir().unwrap();
    let path = d.path().join("stringy.nc");
    {
        let mut file = netcdf::create(path.clone()).unwrap();
        file.add_string_variable("s", &[]).unwrap();

        let mut group = file.add_group("g").unwrap();
        group.add_string_variable("str", &[]).unwrap();
    }
    {
        let file = netcdf::open(path).unwrap();
        let var = file.variable("s").unwrap();
        assert_eq!(var.vartype(), NcVariableType::String);
        let var = file.variable("g/str").unwrap();
        assert_eq!(var.vartype(), NcVariableType::String);
    }
}
