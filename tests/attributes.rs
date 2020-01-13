use netcdf::attribute::AttrValue;

mod common;
use common::test_location;

#[test]
fn attributes_read() {
    let f = test_location().join("patmosx_v05r03-preliminary_NOAA-19_asc_d20130630_c20140325.nc");
    let file = netcdf::open(&f).unwrap();

    let attr = &file
        .attribute("PROGLANG")
        .expect("netcdf error")
        .expect("Could not find attribute");

    assert_eq!(attr.name(), "PROGLANG");

    for attr in file.attributes().unwrap() {
        let attr = attr.unwrap();
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
/// Making sure attributes are updated correctly (replacing previous value)
fn attribute_put() {
    let d = tempfile::tempdir().expect("Could not create tempdir");
    let p = d.path().join("attribute_put.nc");
    let mut f = netcdf::create(p).unwrap();

    f.add_attribute("a", "1").unwrap();
    assert_eq!(
        f.attribute("a").unwrap().unwrap().value().unwrap(),
        "1".into()
    );
    f.add_attribute("b", "2").unwrap();
    assert_eq!(
        f.attribute("b").unwrap().unwrap().value().unwrap(),
        "2".into()
    );
    f.add_attribute("a", 2u32).unwrap();
    assert_eq!(
        f.attribute("a").unwrap().unwrap().value().unwrap(),
        2u32.into()
    );
    f.add_attribute("b", "2").unwrap();
    assert_eq!(
        f.attribute("b").unwrap().unwrap().value().unwrap(),
        "2".into()
    );
}
#[test]
#[cfg(feature = "ndarray")]
fn open_pres_temp_4d() {
    let f = test_location().join("pres_temp_4D.nc");

    let file = netcdf::open(&f).unwrap();

    let pres = &file.variable("pressure").unwrap().unwrap();
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
fn global_attrs() {
    let f = test_location().join("patmosx_v05r03-preliminary_NOAA-19_asc_d20130630_c20140325.nc");

    let file = netcdf::open(&f).unwrap();

    let ch1_attr = &file
        .attribute("CH1_DARK_COUNT")
        .expect("netcdf error")
        .expect("Could not find attribute");
    let chi = ch1_attr.value().unwrap();
    let eps = 1e-6;
    if let AttrValue::Float(x) = chi {
        assert!((x - 40.65863).abs() < eps);
    } else {
        panic!("Did not get the expected attr type");
    }

    let sensor_attr = &file
        .attribute("sensor")
        .expect("netcdf error")
        .expect("Could not find attribute");
    let sensor_data = sensor_attr.value().unwrap();
    if let AttrValue::Str(x) = sensor_data {
        assert_eq!("AVHRR/3", x);
    } else {
        panic!("Did not get the expected attr type");
    }
}
#[test]
fn all_attr_types() {
    let d = tempfile::tempdir().unwrap();
    let u8string = "Testing utf8 with æøå and even 😀";
    {
        let f = d.path().join("all_attr_types.nc");
        let mut file = netcdf::create(&f).unwrap();

        file.add_attribute("attr_byte", 3 as i8).unwrap();
        file.add_attribute("attr_ubyte", 3 as u8).unwrap();
        file.add_attribute("attr_short", 3 as i16).unwrap();
        file.add_attribute("attr_ushort", 3 as u16).unwrap();
        file.add_attribute("attr_int", 3 as i32).unwrap();
        file.add_attribute("attr_uint", 3 as u32).unwrap();
        file.add_attribute("attr_int64", 3 as i64).unwrap();
        file.add_attribute("attr_uint64", 3 as u64).unwrap();
        file.add_attribute("attr_float", 3.2 as f32).unwrap();
        file.add_attribute("attr_double", 3.2 as f64).unwrap();
        file.add_attribute("attr_text", "Hello world!").unwrap();

        file.add_attribute("attr_text_utf8", u8string).unwrap();
    }

    {
        let f = d.path().join("all_attr_types.nc");
        let file = netcdf::open(&f).unwrap();

        assert_eq!(
            AttrValue::Uchar(3),
            file.attribute("attr_ubyte")
                .unwrap()
                .unwrap()
                .value()
                .unwrap()
        );
        assert_eq!(
            AttrValue::Schar(3),
            file.attribute("attr_byte")
                .unwrap()
                .unwrap()
                .value()
                .unwrap()
        );
        assert_eq!(
            AttrValue::Ushort(3),
            file.attribute("attr_ushort")
                .unwrap()
                .unwrap()
                .value()
                .unwrap()
        );
        assert_eq!(
            AttrValue::Short(3),
            file.attribute("attr_short")
                .unwrap()
                .unwrap()
                .value()
                .unwrap()
        );
        assert_eq!(
            AttrValue::Int(3),
            file.attribute("attr_int")
                .unwrap()
                .unwrap()
                .value()
                .unwrap()
        );
        assert_eq!(
            AttrValue::Uint(3),
            file.attribute("attr_uint")
                .unwrap()
                .unwrap()
                .value()
                .unwrap()
        );
        assert_eq!(
            AttrValue::Ulonglong(3),
            file.attribute("attr_uint64")
                .unwrap()
                .unwrap()
                .value()
                .unwrap()
        );
        assert_eq!(
            AttrValue::Longlong(3),
            file.attribute("attr_int64")
                .unwrap()
                .unwrap()
                .value()
                .unwrap()
        );
        assert_eq!(
            AttrValue::Float(3.2),
            file.attribute("attr_float")
                .unwrap()
                .unwrap()
                .value()
                .unwrap()
        );
        assert_eq!(
            AttrValue::Double(3.2),
            file.attribute("attr_double")
                .unwrap()
                .unwrap()
                .value()
                .unwrap()
        );
        assert_eq!(
            AttrValue::Str("Hello world!".into()),
            file.attribute("attr_text")
                .unwrap()
                .unwrap()
                .value()
                .unwrap()
        );
        assert_eq!(
            AttrValue::Str(u8string.into()),
            file.attribute("attr_text_utf8")
                .unwrap()
                .unwrap()
                .value()
                .unwrap()
        );
    }
}

#[test]
fn multi_attributes() {
    let d = tempfile::tempdir().unwrap();
    let path = d.path().join("multi_attributes");
    {
        let mut file = netcdf::create(&path).unwrap();
        file.add_attribute("u8s", vec![1_u8, 2, 3, 4]).unwrap();
        file.add_attribute("i8s", vec![1_i8, 2, 3, 4]).unwrap();
        file.add_attribute("u16s", vec![1_u16, 2, 3, 4]).unwrap();
        file.add_attribute("i16s", vec![1_i16, 2, 3, 4]).unwrap();
        file.add_attribute("u32s", vec![1_u32, 2, 3, 4]).unwrap();
        file.add_attribute("i32s", vec![1_i32, 2, 3, 4]).unwrap();
        file.add_attribute("u64s", vec![1_u64, 2, 3, 4]).unwrap();
        file.add_attribute("i64s", vec![1_i64, 2, 3, 4]).unwrap();
        file.add_attribute("f32s", vec![1.0_f32, 2.0, 3.0, 4.0])
            .unwrap();
        file.add_attribute("f64s", vec![1.0_f64, 2.0, 3.0, 4.0])
            .unwrap();
    }
    let file = netcdf::open(path).unwrap();
    let mut atts = 0;
    for att in file.attributes().unwrap() {
        let att = att.unwrap();
        match att.name() {
            "u8s" => {
                assert_eq!(att.value().unwrap(), vec![1_u8, 2, 3, 4].into());
            }
            "i8s" => {
                assert_eq!(att.value().unwrap(), vec![1_i8, 2, 3, 4].into());
            }
            "u16s" => {
                assert_eq!(att.value().unwrap(), vec![1_u16, 2, 3, 4].into());
            }
            "i16s" => {
                assert_eq!(att.value().unwrap(), vec![1_i16, 2, 3, 4].into());
            }
            "u32s" => {
                assert_eq!(att.value().unwrap(), vec![1_u32, 2, 3, 4].into());
            }
            "i32s" => {
                assert_eq!(att.value().unwrap(), vec![1_i32, 2, 3, 4].into());
            }
            "u64s" => {
                assert_eq!(att.value().unwrap(), vec![1_u64, 2, 3, 4].into());
            }
            "i64s" => {
                assert_eq!(att.value().unwrap(), vec![1_i64, 2, 3, 4].into());
            }
            "f32s" => {
                assert_eq!(att.value().unwrap(), vec![1.0_f32, 2.0, 3.0, 4.0].into());
            }
            "f64s" => {
                assert_eq!(att.value().unwrap(), vec![1.0_f64, 2.0, 3.0, 4.0].into());
            }
            name => panic!("{} not covered", name),
        }
        atts += 1;
    }
    assert_eq!(atts, 10);
}
