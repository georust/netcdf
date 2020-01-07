#[test]
fn dimensions() {
    let d = tempfile::tempdir().unwrap();
    let path = d.path().join("dimensions.rs");

    let mut file = netcdf::create(path).unwrap();

    let mut group = file.add_group("group").unwrap();

    group.add_dimension("a", 5).unwrap();
    group.add_dimension("b", 6).unwrap();
    group.add_unlimited_dimension("c").unwrap();

    let dim = group.dimension("a").unwrap().unwrap();
    assert_eq!(dim.len(), 5);
    let dim = group.dimension("b").unwrap().unwrap();
    assert_eq!(dim.len(), 6);
    let dim = group.dimension("c").unwrap().unwrap();
    assert_eq!(dim.len(), 0);
    assert!(group.dimension("d").unwrap().is_none());
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

    assert_eq!(group.groups().unwrap().count(), 3);
    assert!(group.group("w").unwrap().is_none());
    assert!(group.group_mut("w").unwrap().is_none());
    assert!(group.group_mut("e").unwrap().is_some());
    assert!(group.group("f").unwrap().is_some());
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

    assert_eq!(group.variables_mut().unwrap().count(), 3);
    assert_eq!(group.variables().unwrap().count(), 3);

    let v = group.variable("v").unwrap().unwrap();
    assert_eq!(v.dimensions().iter().count(), 0);
    assert_eq!(v.len(), 1);
    let z = group.variable_mut("z").unwrap().unwrap();
    assert_eq!(z.dimensions()[0].len(), 3);
    assert_eq!(z.vartype(), netcdf_sys::NC_UBYTE);
    assert_eq!(z.name().unwrap(), "z");

    assert!(group.variable("vvvvv").unwrap().is_none());

    for var in group.variables_mut().unwrap() {
        let mut var = var.unwrap();
        var.compression(3).unwrap();
        if var.name().unwrap() == "z" {
            var.chunking(&[1]).unwrap();
        } else {
            var.chunking(&[]).unwrap();
        }
    }
}
