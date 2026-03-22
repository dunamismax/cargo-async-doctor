#[path = "support/mod.rs"]
mod support;

use cargo_async_doctor::{
    cli::{Cli, MessageFormat},
    diagnostics::CheckId,
    explain, render, scan,
};
use serde_json::Value;

fn scan_fixture(name: &str) -> cargo_async_doctor::diagnostics::ScanReport {
    scan_fixture_with_workspace(name, false)
}

fn scan_fixture_with_workspace(
    name: &str,
    workspace: bool,
) -> cargo_async_doctor::diagnostics::ScanReport {
    let manifest_path = support::fixture_root(name).join("Cargo.toml");

    let cli = Cli {
        message_format: MessageFormat::Human,
        command: None,
        workspace,
        manifest_path: Some(manifest_path),
    };

    scan::scan(&cli).unwrap()
}

#[test]
fn placeholder_fixture_is_discoverable() {
    let fixture = support::fixture_root("placeholder/minimal-bin");

    assert!(fixture.join("Cargo.toml").exists());
    assert!(fixture.join("src/main.rs").exists());
}

#[test]
fn placeholder_fixture_scans_cleanly() {
    let manifest_path = support::fixture_root("placeholder/minimal-bin").join("Cargo.toml");
    let cli = Cli {
        message_format: MessageFormat::Human,
        command: None,
        workspace: false,
        manifest_path: Some(manifest_path.clone()),
    };

    let report = scan::scan(&cli).unwrap();

    assert_eq!(
        report.schema_version,
        cargo_async_doctor::diagnostics::SCAN_SCHEMA_VERSION
    );
    assert!(report.diagnostics.is_empty());
    assert_eq!(report.summary.total, 0);
    assert_eq!(report.target.packages.len(), 1);
    assert_eq!(report.target.packages[0].name, "fixture-minimal-bin");
    assert_eq!(
        report.target.manifest_path.as_deref(),
        Some(manifest_path.to_string_lossy().as_ref())
    );
}

#[test]
fn blocking_sleep_positive_fixture_emits_stable_check() {
    let report = scan_fixture("phase2/blocking-sleep-positive");

    assert_eq!(report.summary.warnings, 1);
    assert_eq!(report.diagnostics.len(), 1);
    assert_eq!(report.diagnostics[0].id, CheckId::BlockingSleepInAsync);
    assert!(report.diagnostics[0].message.contains("thread::sleep"));
    assert_eq!(report.diagnostics[0].location.file_path, "src/main.rs");
    assert_eq!(report.diagnostics[0].location.line, Some(4));
    assert!(report.diagnostics[0]
        .help
        .as_deref()
        .is_some_and(|help| help.contains("tokio::time::sleep")));
}

#[test]
fn blocking_sleep_negative_fixture_stays_quiet() {
    let report = scan_fixture("phase2/blocking-sleep-negative");

    assert!(report.diagnostics.is_empty());
    assert_eq!(report.summary.warnings, 0);
}

#[test]
fn blocking_std_api_positive_fixture_emits_stable_check() {
    let report = scan_fixture("phase2/blocking-std-api-positive");

    assert_eq!(report.summary.warnings, 1);
    assert_eq!(report.diagnostics.len(), 1);
    assert_eq!(report.diagnostics[0].id, CheckId::BlockingStdApiInAsync);
    assert!(report.diagnostics[0].message.contains("fs::read_to_string"));
    assert_eq!(report.diagnostics[0].location.file_path, "src/main.rs");
    assert_eq!(report.diagnostics[0].location.line, Some(4));
    assert!(report.diagnostics[0]
        .help
        .as_deref()
        .is_some_and(|help| help.contains("tokio::fs")));
}

#[test]
fn blocking_std_api_negative_fixture_filters_sync_and_lookalike_calls() {
    let report = scan_fixture("phase2/blocking-std-api-negative");

    assert!(report.diagnostics.is_empty());
    assert_eq!(report.summary.warnings, 0);
}

#[test]
fn sync_async_bridge_positive_fixture_emits_stable_check() {
    let report = scan_fixture("phase2/sync-async-bridge-positive");

    assert_eq!(report.summary.warnings, 1);
    assert_eq!(report.diagnostics.len(), 1);
    assert_eq!(report.diagnostics[0].id, CheckId::SyncAsyncBridgeHazard);
    assert!(report.diagnostics[0]
        .message
        .contains("Handle::current().block_on"));
    assert_eq!(report.diagnostics[0].location.file_path, "src/main.rs");
    assert_eq!(report.diagnostics[0].location.line, Some(4));
    assert!(report.diagnostics[0]
        .help
        .as_deref()
        .is_some_and(|help| help.contains("call chain async")));
}

#[test]
fn sync_async_bridge_negative_fixture_filters_local_lookalikes() {
    let report = scan_fixture("phase2/sync-async-bridge-negative");

    assert!(report.diagnostics.is_empty());
    assert_eq!(report.summary.warnings, 0);
}

#[test]
fn workspace_root_package_fixture_scans_only_root_package_by_default() {
    let report = scan_fixture("phase4/workspace-root-package");

    assert_eq!(report.summary.warnings, 1);
    assert_eq!(report.target.packages.len(), 1);
    assert_eq!(report.target.packages[0].name, "workspace-root-package");
    assert_eq!(report.diagnostics[0].package.name, "workspace-root-package");
    assert_eq!(report.diagnostics[0].location.file_path, "src/main.rs");
    assert_eq!(report.diagnostics[0].location.line, Some(4));
}

#[test]
fn workspace_member_manifest_scans_only_that_member() {
    let manifest_path = support::fixture_root("phase4/workspace-root-package")
        .join("member-bin")
        .join("Cargo.toml");
    let cli = Cli {
        message_format: MessageFormat::Human,
        command: None,
        workspace: false,
        manifest_path: Some(manifest_path),
    };

    let report = scan::scan(&cli).unwrap();

    assert_eq!(report.summary.warnings, 1);
    assert_eq!(report.target.packages.len(), 1);
    assert_eq!(report.target.packages[0].name, "member-bin");
    assert_eq!(report.diagnostics[0].package.name, "member-bin");
    assert_eq!(
        report.diagnostics[0].location.file_path,
        "member-bin/src/main.rs"
    );
    assert_eq!(report.diagnostics[0].id, CheckId::BlockingStdApiInAsync);
}

#[test]
fn workspace_flag_scans_all_workspace_members_and_root_package() {
    let report = scan_fixture_with_workspace("phase4/workspace-root-package", true);

    assert_eq!(report.summary.warnings, 3);
    assert_eq!(
        report
            .target
            .packages
            .iter()
            .map(|package| package.name.as_str())
            .collect::<Vec<_>>(),
        vec!["member-lib", "member-bin", "workspace-root-package"]
    );
    assert_eq!(
        report
            .diagnostics
            .iter()
            .map(|diagnostic| diagnostic.package.name.as_str())
            .collect::<Vec<_>>(),
        vec!["member-bin", "member-lib", "workspace-root-package"]
    );
    assert_eq!(
        report
            .diagnostics
            .iter()
            .map(|diagnostic| diagnostic.location.file_path.as_str())
            .collect::<Vec<_>>(),
        vec![
            "member-bin/src/main.rs",
            "member-lib/src/lib.rs",
            "src/main.rs"
        ]
    );
}

#[test]
fn virtual_workspace_manifest_scans_default_members_without_workspace_flag() {
    let report = scan_fixture("phase4/virtual-workspace");

    assert_eq!(report.summary.warnings, 1);
    assert_eq!(report.target.packages.len(), 1);
    assert_eq!(report.target.packages[0].name, "default-member");
    assert_eq!(report.diagnostics[0].package.name, "default-member");
    assert_eq!(
        report.diagnostics[0].location.file_path,
        "default-member/src/main.rs"
    );
    assert!(report
        .notes
        .iter()
        .any(|note| note.contains("default workspace members only")));
}

#[test]
fn stray_rust_files_under_src_are_not_scanned_without_module_reachability() {
    let report = scan_fixture("phase5/uncompiled-src-file");

    assert!(report.diagnostics.is_empty());
    assert_eq!(report.summary.warnings, 0);
    assert_eq!(report.target.packages.len(), 1);
    assert_eq!(report.target.packages[0].name, "uncompiled-src-file");
}

#[test]
fn custom_target_paths_outside_src_are_scanned_via_cargo_metadata() {
    let report = scan_fixture("phase5/custom-target-path");

    assert_eq!(report.summary.warnings, 1);
    assert_eq!(report.diagnostics.len(), 1);
    assert_eq!(report.diagnostics[0].id, CheckId::BlockingSleepInAsync);
    assert_eq!(report.diagnostics[0].location.file_path, "bin/helper.rs");
    assert_eq!(report.diagnostics[0].location.line, Some(4));
}

#[test]
fn nested_inline_modules_do_not_inherit_parent_aliases_from_outer_scope() {
    let report = scan_fixture("phase5/nested-inline-module-alias-leakage");

    assert!(report.diagnostics.is_empty());
    assert_eq!(report.summary.warnings, 0);
    assert_eq!(report.target.packages.len(), 1);
    assert_eq!(
        report.target.packages[0].name,
        "nested-inline-module-alias-leakage"
    );
}

#[test]
fn cfg_disabled_code_and_modules_are_not_scanned() {
    let report = scan_fixture("phase6/cfg-reachability");

    assert_eq!(report.summary.warnings, 1);
    assert_eq!(report.diagnostics.len(), 1);
    assert_eq!(report.diagnostics[0].id, CheckId::BlockingSleepInAsync);
    assert_eq!(
        report.diagnostics[0].location.file_path,
        "src/enabled_module.rs"
    );
    assert_eq!(report.diagnostics[0].location.line, Some(4));
}

#[test]
fn nested_inline_path_attributes_follow_rustc_resolution() {
    let report = scan_fixture("phase6/nested-inline-path-attribute");

    assert_eq!(report.summary.warnings, 1);
    assert_eq!(report.diagnostics.len(), 1);
    assert_eq!(report.diagnostics[0].id, CheckId::BlockingSleepInAsync);
    assert_eq!(
        report.diagnostics[0].location.file_path,
        "src/outer/child.rs"
    );
    assert_eq!(report.diagnostics[0].location.line, Some(4));
}

#[test]
fn json_output_for_workspace_fixture_is_structured() {
    let report = scan_fixture_with_workspace("phase4/workspace-root-package", true);
    let rendered = render::render_scan_report(MessageFormat::Json, &report).unwrap();
    let value: Value = serde_json::from_str(&rendered).unwrap();

    assert_eq!(value["schema_version"], 1);
    assert!(value.get("placeholder").is_none());
    assert_eq!(value["summary"]["warnings"], 3);
    assert_eq!(
        value["diagnostics"][0]["package"]["name"],
        Value::String("member-bin".to_string())
    );
    assert_eq!(
        value["diagnostics"][0]["location"]["file_path"],
        Value::String("member-bin/src/main.rs".to_string())
    );
    assert_eq!(value["diagnostics"][0]["location"]["line"], 4);
    assert_eq!(
        value["diagnostics"][0]["id"],
        Value::String(CheckId::BlockingStdApiInAsync.as_str().to_string())
    );
    assert_eq!(
        value["diagnostics"][0]["severity"],
        Value::String("warning".to_string())
    );
}

#[test]
fn explain_reports_known_phase_two_check_in_json() {
    let report = explain::explain("sync-async-bridge-hazard");
    let rendered = render::render_explain_report(MessageFormat::Json, &report).unwrap();
    let value: Value = serde_json::from_str(&rendered).unwrap();

    assert_eq!(value["found"], true);
    assert_eq!(value["requested_check_id"], "sync-async-bridge-hazard");
    assert_eq!(value["explanation"]["id"], "sync-async-bridge-hazard");
    assert!(value["explanation"]["references"].is_array());
}

#[test]
fn explain_reports_unknown_check_id_in_json() {
    let report = explain::explain("unknown-check-id");
    let rendered = render::render_explain_report(MessageFormat::Json, &report).unwrap();
    let value: Value = serde_json::from_str(&rendered).unwrap();

    assert_eq!(value["found"], false);
    assert_eq!(value["error"], "unknown-check-id");
    assert_eq!(value["explanation"], Value::Null);
}
