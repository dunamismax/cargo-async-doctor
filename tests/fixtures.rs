#[path = "support/mod.rs"]
mod support;

use cargo_async_doctor::{
    cli::{Cli, MessageFormat},
    diagnostics::CheckId,
    explain, render, scan,
};
use serde_json::Value;

fn scan_fixture(name: &str) -> cargo_async_doctor::diagnostics::ScanReport {
    let manifest_path = support::fixture_root(name).join("Cargo.toml");

    let cli = Cli {
        message_format: MessageFormat::Human,
        command: None,
        workspace: false,
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

    assert!(!report.placeholder);
    assert!(report.diagnostics.is_empty());
    assert_eq!(report.summary.total, 0);
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
fn json_output_for_phase_two_fixture_is_structured() {
    let report = scan_fixture("phase2/blocking-std-api-positive");
    let rendered = render::render_scan_report(MessageFormat::Json, &report).unwrap();
    let value: Value = serde_json::from_str(&rendered).unwrap();

    assert_eq!(value["schema_version"], 1);
    assert_eq!(value["placeholder"], false);
    assert_eq!(value["summary"]["warnings"], 1);
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
