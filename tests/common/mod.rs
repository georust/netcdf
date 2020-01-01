/// Get location of the test files
pub(crate) fn test_location() -> std::path::PathBuf {
    use std::path::Path;

    let mnf_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".into());
    Path::new(&mnf_dir).join("tests").join("testdata")
}

