#[test]
fn create_classic_model() {
    let d = tempfile::tempdir().unwrap();
    let path = d.path().join("create_classic.nc");

    let mut file = netcdf::create_with(path, netcdf::Options::CLASSIC).unwrap();
    // Classic mode does not support groups
    file.add_group("grp").unwrap_err();
}

#[test]
fn create_with_options() {
    let d = tempfile::tempdir().unwrap();
    let path0 = d.path().join("create0.nc");
    let path1 = d.path().join("create1.nc");

    let mut file = netcdf::create_with(path0, netcdf::Options::_64BIT_DATA).unwrap();
    file.add_group("grp").unwrap_err();
    let mut file = netcdf::create_with(path1, netcdf::Options::_64BIT_OFFSET).unwrap();
    file.add_group("grp").unwrap_err();
}

#[test]
fn noclobber() {
    let d = tempfile::tempdir().unwrap();
    let path = d.path().join("cdf5.nc");
    {
        let mut file = netcdf::create_with(&path, netcdf::Options::_64BIT_DATA).unwrap();
        file.add_dimension("t", 1).unwrap();
    }
    let _file = netcdf::create_with(&path, netcdf::Options::NOCLOBBER).unwrap_err();
}

#[test]
fn appending_with() {
    let d = tempfile::tempdir().unwrap();
    let path = d.path().join("cdf5.nc");
    {
        let mut file = netcdf::create_with(&path, netcdf::Options::NETCDF4).unwrap();
        file.add_dimension("t", 1).unwrap();
    }
    let _file =
        netcdf::append_with(&path, netcdf::Options::NETCDF4 | netcdf::Options::DISKLESS).unwrap();
}

#[test]
fn fetch_from_path() {
    let d = tempfile::tempdir().unwrap();
    let path = d.path().join("cdf5.nc");
    {
        let mut file = netcdf::create(path.clone()).unwrap();
        let mut group = file.add_group("grp").unwrap();
        let mut subgroup = group.add_group("subgrp").unwrap();
        subgroup.add_dimension("dim", 1).unwrap();
        subgroup.add_variable::<f64>("var", &["dim"]).unwrap();
        subgroup.add_attribute("attr", "test").unwrap();
    }
    let file = netcdf::open(path).unwrap();
    assert_eq!(
        file.group("grp/subgrp")
            .unwrap()
            .unwrap()
            .variable("var")
            .unwrap()
            .name(),
        file.variable("grp/subgrp/var").unwrap().name(),
    );
    match file.attribute("grp/subgrp/attr").unwrap().value().unwrap() {
        netcdf::AttributeValue::Str(string) => assert_eq!(string, "test"),
        _ => panic!(),
    }
}
