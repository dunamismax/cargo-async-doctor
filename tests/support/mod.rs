use std::path::{Path, PathBuf};

pub fn repo_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

pub fn fixture_root(name: &str) -> PathBuf {
    repo_root().join("fixtures").join(name)
}
