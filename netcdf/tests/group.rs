#[test]
fn dimensions() {
    let d = tempfile::tempdir().unwrap();
    let path = d.path().join("dimensions.rs");

    let mut file = netcdf::create(path).unwrap();

    let mut group = file.add_group("group").unwrap();

    group.add_dimension("a", 5).unwrap();
    group.add_dimension("b", 6).unwrap();
    group.add_unlimited_dimension("c").unwrap();

    let dim = group.dimension("a").unwrap();
    assert_eq!(dim.len(), 5);
    let dim = group.dimension("b").unwrap();
    assert_eq!(dim.len(), 6);
    let dim = group.dimension("c").unwrap();
    assert_eq!(dim.len(), 0);
    assert!(group.dimension("d").is_none());
}

#[test]
fn groups() {
    let d = tempfile::tempdir().unwrap();
    let path = d.path().join("groups.rs");
    let mut file = netcdf::create(path).unwrap();
    let mut group = file.add_group("group").unwrap();
    group.add_group("g").unwrap();
    group.add_group("e").unwrap();
    group.add_group("f").unwrap();

    assert_eq!(group.groups().count(), 3);
    assert!(group.group("w").is_none());
    assert!(group.group_mut("w").is_none());
    assert!(group.group_mut("e").is_some());
    assert!(group.group("f").is_some());
}

#[test]
fn find_variable() {
    let d = tempfile::tempdir().unwrap();
    let path = d.path().join("groups.rs");
    let mut file = netcdf::create(path).unwrap();
    let mut group = file.add_group("group").unwrap();

    group.add_variable::<u8>("v", &[]).unwrap();
    group.add_variable::<u8>("w", &[]).unwrap();
    group.add_dimension("d", 3).unwrap();
    group.add_variable::<u8>("z", &["d"]).unwrap();

    assert_eq!(group.variables_mut().count(), 3);
    assert_eq!(group.variables().count(), 3);

    let v = group.variable("v").unwrap();
    assert_eq!(v.dimensions().iter().count(), 0);
    assert_eq!(v.len(), 1);
    let z = group.variable_mut("z").unwrap();
    assert_eq!(z.dimensions()[0].len(), 3);
    assert_eq!(
        z.vartype(),
        netcdf::types::NcVariableType::Int(netcdf::types::IntType::U8)
    );
    assert_eq!(z.name(), "z");

    assert!(group.variable("vvvvv").is_none());

    for mut var in group.variables_mut() {
        assert_eq!(var.chunking().unwrap(), None);
        if !var.dimensions().is_empty() {
            var.set_compression(3, false).unwrap();
        }
        if var.name() == "z" {
            var.set_chunking(&[1]).unwrap();
            assert_eq!(var.chunking().unwrap(), Some(vec![1]));
        } else {
            var.set_chunking(&[]).unwrap();
            assert_eq!(var.chunking().unwrap(), None);
        }
    }
}

#[test]
fn add_and_get_from_path() {
    let d = tempfile::tempdir().unwrap();
    let path = d.path().join("cdf5.nc");
    {
        let mut file = netcdf::create(path.clone()).unwrap();
        file.add_group("a/b").unwrap();
        file.add_dimension("a/b/dim", 1).unwrap();
        assert!(file.add_dimension("a/c/dim", 1).is_err());
        file.add_variable::<f64>("a/b/var", &["dim"]).unwrap();
        assert!(file.add_variable::<f64>("a/c/var", &["dim"]).is_err());
        file.add_attribute("a/b/attr", "test").unwrap();
        assert!(file.add_attribute("a/c/test", "test").is_err());
    }
    let file = netcdf::open(path).unwrap();
    let root = file.root().unwrap();
    assert_eq!(
        root.group("a/b").unwrap().variable("var").unwrap().name(),
        root.variable("a/b/var").unwrap().name(),
    );
    assert!(file.group("missing/subgrp").is_err());
    match file.attribute("a/b/attr").unwrap().value().unwrap() {
        netcdf::AttributeValue::Str(string) => assert_eq!(string, "test"),
        _ => panic!(),
    }
}
