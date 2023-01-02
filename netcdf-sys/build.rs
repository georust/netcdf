use std::path::{Path, PathBuf};
use std::process::Command;

use semver::Version;

macro_rules! feature {
    ($feature:expr) => {
        std::env::var(concat!("CARGO_FEATURE_", $feature))
    };
}

#[derive(Debug)]
struct NcMetaHeader {
    version: Version,

    has_nc2: bool,
    has_nc4: bool,
    has_hdf4: bool,
    has_hdf5: bool,
    has_szip: bool,
    has_szip_write: bool,
    has_dap2: bool,
    has_dap4: bool,
    has_byterange: bool,
    has_diskless: bool,
    has_mmap: bool,
    has_jna: bool,
    has_pnetcdf: bool,
    has_parallel4: bool,
    has_parallel: bool,

    has_cdf5: bool,
    has_erange_fill: bool,
    relax_coord_bound: bool,
    dispatch_version: Option<u8>,
    has_par_filters: bool,
    has_nczarr: bool,
    has_multifilters: bool,
    has_logging: bool,
    has_quantize: bool,
    has_zstd: bool,
    has_benchmarks: bool,
}

impl NcMetaHeader {
    fn gather_from_includeheader(path: &std::path::Path) -> Self {
        macro_rules! match_prefix {
            ($line: expr, $prefix: expr, $item: expr) => {
                if let Some(item) = match_prefix_bool($line, $prefix) {
                    $item = item;
                }
            };
        }
        fn match_prefix<'a>(line: &'a str, prefix: &str) -> Option<&'a str> {
            line.strip_prefix(&format!("#define {prefix} "))
                .map(|item| item.trim())
        }
        fn match_prefix_bool(line: &str, prefix: &str) -> Option<bool> {
            match_prefix(line, prefix).map(|item| item.starts_with('1'))
        }
        let meta = std::fs::read_to_string(path).expect("Could not read header file");

        let mut info = Self {
            version: Version::new(0, 0, 0),
            has_nc2: false,
            has_nc4: false,
            has_benchmarks: false,
            has_byterange: false,
            has_cdf5: false,
            has_dap2: false,
            has_dap4: false,
            has_diskless: false,
            dispatch_version: None,
            has_erange_fill: false,
            has_hdf4: false,
            has_hdf5: false,
            has_jna: false,
            has_logging: false,
            has_mmap: false,
            has_multifilters: false,
            has_nczarr: false,
            has_par_filters: false,
            has_parallel: false,
            has_parallel4: false,
            has_pnetcdf: false,
            has_quantize: false,
            has_szip: false,
            has_szip_write: false,
            has_zstd: false,
            relax_coord_bound: false,
        };
        for line in meta.lines() {
            if let Some(ncversion) = match_prefix(line, "NC_VERSION") {
                info.version = Version::parse(ncversion.trim_matches('"')).unwrap();
            }
            if let Some(dversion) = match_prefix(line, "NC_DISPATCH_VERSION") {
                let (dversion, _) = dversion.split_once(' ').unwrap();
                info.dispatch_version = Some(dversion.parse().unwrap());
            }
            match_prefix!(line, "NC_HAS_NC2", info.has_nc2);
            match_prefix!(line, "NC_HAS_NC4", info.has_nc4);
            match_prefix!(line, "NC_HAS_HDF4", info.has_hdf4);
            match_prefix!(line, "NC_HAS_HDF5", info.has_hdf5);
            match_prefix!(line, "NC_HAS_SZIP", info.has_szip);
            match_prefix!(line, "NC_HAS_DAP2", info.has_dap2);
            match_prefix!(line, "NC_HAS_DAP4", info.has_dap4);
            match_prefix!(line, "NC_HAS_DISKLESS", info.has_diskless);
            match_prefix!(line, "NC_HAS_MMAP", info.has_mmap);
            match_prefix!(line, "NC_HAS_JNA", info.has_jna);
            match_prefix!(line, "NC_HAS_PNETCDF", info.has_pnetcdf);
            match_prefix!(line, "NC_HAS_PARALLEL", info.has_parallel);
            match_prefix!(line, "NC_HAS_CDF5", info.has_cdf5);
            match_prefix!(line, "NC_HAS_BYTERANGE", info.has_byterange);
            match_prefix!(line, "NC_HAS_BENCHMARKS", info.has_benchmarks);
            match_prefix!(line, "NC_HAS_ERANGE_FILL", info.has_erange_fill);
            match_prefix!(line, "NC_HAS_ZSTD", info.has_zstd);
            match_prefix!(line, "NC_HAS_QUANTIZE", info.has_quantize);
            match_prefix!(line, "NC_HAS_LOGGING", info.has_logging);
            match_prefix!(line, "NC_HAS_MULTIFILTERS", info.has_multifilters);
            match_prefix!(line, "NC_HAS_NCZARR", info.has_nczarr);
            match_prefix!(line, "NC_HAS_PAR_FILTERS", info.has_par_filters);
            match_prefix!(line, "NC_RELAX_COORD_BOUND", info.relax_coord_bound);
            match_prefix!(line, "NC_HAS_PARALLEL", info.has_parallel);
            match_prefix!(line, "NC_HAS_PARALLEL4", info.has_parallel4);
            match_prefix!(line, "NC_HAS_SZIP_WRITE", info.has_szip_write);
        }
        info
    }

    fn emit_feature_flags(&self) {
        if self.has_dap2 || self.has_dap4 {
            println!("cargo:rustc-cfg=feature=\"has-dap\"");
            println!("cargo:has-dap=1");
        } else {
            assert!(
                feature!("DAP").is_err(),
                "DAP requested but not found in this installation of netCDF"
            );
        }
        if self.has_mmap {
            println!("cargo:rustc-cfg=feature=\"has-mmap\"");
            println!("cargo:has-mmap=1");
        } else {
            assert!(
                feature!("MEMIO").is_err(),
                "MEMIO requested but not found in this installation of netCDF"
            );
        }
    }
}

#[derive(Debug)]
struct NcInfo {
    version: Option<Version>,

    includedir: PathBuf,
    libdir: PathBuf,
    libname: String,
}

fn from_utf8_to_trimmed_string(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).trim().to_owned()
}

impl NcInfo {
    fn guess() -> Self {
        todo!()
    }
    fn from_path(path: &Path) -> Self {
        Self {
            version: None,
            includedir: path.join("include"),
            libdir: path.join("lib"),
            libname: "netcdf".to_owned(),
        }
    }
    fn gather_from_ncconfig(search_path: Option<&Path>) -> Option<Self> {
        let path = if let Some(search_path) = search_path {
            let search_path = search_path.join("bin").join("nc-config");
            search_path.as_os_str().to_owned()
        } else {
            std::ffi::OsString::from("nc-config")
        };
        let cmd = || Command::new(&path);
        cmd().arg("--help").status().ok()?;

        let extract = |arg: &str| -> Result<Option<String>, Box<dyn std::error::Error>> {
            let output = &cmd().arg(arg).output()?;
            if output.status.success() {
                Ok(Some(from_utf8_to_trimmed_string(&output.stdout)))
            } else {
                Ok(None)
            }
        };

        let version = if let Ok(Some(version)) = extract("--version") {
            version.strip_prefix("netCDF ").unwrap().to_owned()
        } else {
            panic!("Could not get information from this installation of NetCDF");
        };
        let version = Version::parse(&version).unwrap();

        let includedir = PathBuf::from(extract("--includedir").unwrap().unwrap());
        let libdir = PathBuf::from(extract("--libdir").unwrap().unwrap());
        let libs = extract("--libs").unwrap().unwrap();
        assert!(libs.contains("-lnetcdf"));
        let libname = "netcdf".to_owned();

        let _inc = std::fs::read_to_string(std::path::Path::new(&includedir).join("netcdf.h"))
            .expect("Could not find netcdf.h");

        Some(Self {
            version: Some(version),
            includedir,
            libdir,
            libname,
        })
    }
}

fn _check_consistent_version_linked() {
    // use libloading
    todo!()
}

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    let info;
    if feature!("STATIC").is_ok() {
        let netcdf_lib = std::env::var("DEP_NETCDFSRC_LIB").unwrap();
        let netcdf_path = PathBuf::from(std::env::var_os("DEP_NETCDFSRC_SEARCH").unwrap());

        info = NcInfo::gather_from_ncconfig(Some(&netcdf_path.join("..")))
            .unwrap_or_else(|| NcInfo::from_path(&netcdf_path.join("..")));

        println!("cargo:rustc-link-search=native={}", netcdf_path.display());
        println!("cargo:rustc-link-lib=static={netcdf_lib}");
    } else {
        println!("cargo:rerun-if-env-changed=NETCDF_DIR");

        let nc_dir = std::env::var_os("NETCDF_DIR")
            .or_else(|| std::env::var_os("NetCDF_DIR"))
            .map(PathBuf::from);

        #[cfg(windows)]
        let nc_dir = nc_dir.map(|d| d.join("Library"));

        info = if let Some(nc_dir) = nc_dir.as_ref() {
            NcInfo::gather_from_ncconfig(Some(nc_dir)).unwrap_or_else(|| NcInfo::from_path(nc_dir))
        } else {
            NcInfo::gather_from_ncconfig(None).unwrap_or_else(NcInfo::guess)
        };

        println!("cargo:rustc-link-search={}/lib", info.libdir.display());
        println!("cargo:rustc-link-lib={}", &info.libname);
    }

    let metaheader = NcMetaHeader::gather_from_includeheader(
        &std::path::Path::new(&info.includedir).join("netcdf_meta.h"),
    );
    if let Some(version) = info.version {
        assert_eq!(version, metaheader.version, "Version mismatch");
    }

    // panic!("{:?}", info);
    // Emit nc flags
    println!("cargo:includedir={}", info.includedir.display());
    println!("cargo:nc_version={}", metaheader.version);
    let versions = [
        Version::new(4, 4, 0),
        Version::new(4, 4, 1),
        Version::new(4, 5, 0),
        Version::new(4, 6, 0),
        Version::new(4, 6, 1),
        Version::new(4, 6, 2),
        Version::new(4, 6, 3),
        Version::new(4, 7, 0),
        Version::new(4, 7, 1),
        Version::new(4, 7, 2),
        Version::new(4, 7, 3),
        Version::new(4, 7, 4),
        Version::new(4, 8, 0),
        Version::new(4, 8, 1),
    ];

    if !versions.contains(&metaheader.version) {
        if versions
            .iter()
            .any(|x| (x.major == metaheader.version.major) && (x.minor == metaheader.version.minor))
        {
            println!("We don't know this release, but it is just a patch difference")
        } else if versions.iter().any(|x| x.major == metaheader.version.major) {
            eprintln!("This minor version of netCDF is not known, but the major version is known and the release is unlikely to contain breaking API changes");
        } else {
            eprintln!("This major version is not known, please file an issue if breaking API changes have been made to netCDF-c");
        }
    }

    for version in versions {
        if metaheader.version >= version {
            println!(
                "cargo:rustc-cfg=feature=\"{}.{}.{}\"",
                version.major, version.minor, version.patch
            );
        }
    }
    metaheader.emit_feature_flags();
}
