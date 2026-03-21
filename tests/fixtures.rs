#[path = "support/mod.rs"]
mod support;

use cargo_async_doctor::{
    cli::{Cli, MessageFormat},
    scan,
};

#[test]
fn placeholder_fixture_is_discoverable() {
    let fixture = support::fixture_root("placeholder/minimal-bin");

    assert!(fixture.join("Cargo.toml").exists());
    assert!(fixture.join("src/main.rs").exists());
}

#[test]
fn placeholder_scan_accepts_fixture_manifest_path() {
    let manifest_path = support::fixture_root("placeholder/minimal-bin").join("Cargo.toml");

    let cli = Cli {
        message_format: MessageFormat::Human,
        workspace: false,
        manifest_path: Some(manifest_path.clone()),
    };

    let report = scan::scan(&cli);

    assert!(report.placeholder);
    assert!(report.diagnostics.is_empty());
    assert_eq!(report.summary.total, 0);
    assert_eq!(
        report.target.manifest_path.as_deref(),
        Some(manifest_path.to_string_lossy().as_ref())
    );
}
