use std::str::FromStr;

use serde::Serialize;

pub const SCAN_SCHEMA_VERSION: u32 = 1;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum CheckId {
    BlockingSleepInAsync,
    BlockingStdApiInAsync,
    SyncAsyncBridgeHazard,
    GuardAcrossAwait,
}

impl CheckId {
    pub const ALL: [Self; 4] = [
        Self::BlockingSleepInAsync,
        Self::BlockingStdApiInAsync,
        Self::SyncAsyncBridgeHazard,
        Self::GuardAcrossAwait,
    ];

    pub const PHASE_TWO_SHIPPED: [Self; 3] = [
        Self::BlockingSleepInAsync,
        Self::BlockingStdApiInAsync,
        Self::SyncAsyncBridgeHazard,
    ];

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::BlockingSleepInAsync => "blocking-sleep-in-async",
            Self::BlockingStdApiInAsync => "blocking-std-api-in-async",
            Self::SyncAsyncBridgeHazard => "sync-async-bridge-hazard",
            Self::GuardAcrossAwait => "guard-across-await",
        }
    }

    pub const fn title(self) -> &'static str {
        match self {
            Self::BlockingSleepInAsync => "Blocking sleep in async contexts",
            Self::BlockingStdApiInAsync => "Blocking std::fs APIs in async contexts",
            Self::SyncAsyncBridgeHazard => "Sync/async bridge hazards in async contexts",
            Self::GuardAcrossAwait => "Guard held across .await",
        }
    }

    pub const fn explanation(self) -> &'static str {
        match self {
            Self::BlockingSleepInAsync => {
                "std::thread::sleep blocks the current OS thread instead of yielding back to the async runtime."
            }
            Self::BlockingStdApiInAsync => {
                "Synchronous std::fs calls perform blocking filesystem I/O on the current thread instead of yielding to the async runtime."
            }
            Self::SyncAsyncBridgeHazard => {
                "Calling Tokio block_on from async code reintroduces a synchronous wait at the runtime boundary and can panic or stall progress."
            }
            Self::GuardAcrossAwait => {
                "Holding a non-async-aware guard across .await can deadlock or block progress."
            }
        }
    }

    pub const fn help(self) -> Option<&'static str> {
        match self {
            Self::BlockingSleepInAsync => Some(
                "For Tokio code, prefer tokio::time::sleep(...).await. If the work must block, move it behind tokio::task::spawn_blocking or another synchronous boundary.",
            ),
            Self::BlockingStdApiInAsync => Some(
                "For Tokio code, prefer tokio::fs when the surrounding code is already async. If you must keep std::fs, isolate it behind tokio::task::spawn_blocking or another synchronous boundary.",
            ),
            Self::SyncAsyncBridgeHazard => Some(
                "Keep the call chain async. If a synchronous bridge is truly required, move it to a clearly synchronous entry point instead of nesting block_on inside async code.",
            ),
            Self::GuardAcrossAwait => None,
        }
    }

    pub fn from_str_name(input: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|id| id.as_str() == input)
    }
}

impl FromStr for CheckId {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_str_name(s).ok_or(())
    }
}

impl std::fmt::Display for CheckId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Warning,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
pub struct DiagnosticPackage {
    pub name: String,
    pub manifest_path: String,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
pub struct DiagnosticLocation {
    pub file_path: String,
    pub package_path: String,
    pub line: Option<usize>,
    pub column: Option<usize>,
    pub end_line: Option<usize>,
    pub end_column: Option<usize>,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
pub struct Diagnostic {
    pub id: CheckId,
    pub severity: Severity,
    pub package: DiagnosticPackage,
    pub location: DiagnosticLocation,
    pub message: String,
    pub help: Option<String>,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
pub struct ScanPackageTarget {
    pub name: String,
    pub manifest_path: String,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
pub struct ScanTarget {
    pub workspace: bool,
    pub manifest_path: Option<String>,
    pub workspace_root: Option<String>,
    pub packages: Vec<ScanPackageTarget>,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
pub struct ScanSummary {
    pub total: usize,
    pub warnings: usize,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
pub struct ScanReport {
    pub schema_version: u32,
    pub target: ScanTarget,
    pub summary: ScanSummary,
    pub diagnostics: Vec<Diagnostic>,
    pub notes: Vec<String>,
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::CheckId;

    #[test]
    fn check_ids_are_stable_and_unique() {
        let ids: Vec<&str> = CheckId::ALL.into_iter().map(CheckId::as_str).collect();
        let unique: HashSet<&str> = ids.iter().copied().collect();

        assert_eq!(ids.len(), unique.len());
        assert_eq!(
            ids,
            vec![
                "blocking-sleep-in-async",
                "blocking-std-api-in-async",
                "sync-async-bridge-hazard",
                "guard-across-await",
            ]
        );
    }

    #[test]
    fn phase_two_checks_have_explanations_and_help() {
        for id in CheckId::PHASE_TWO_SHIPPED {
            assert!(!id.title().is_empty());
            assert!(!id.explanation().is_empty());
            assert!(id.help().is_some());
        }
    }

    #[test]
    fn parses_stable_check_ids_from_strings() {
        assert_eq!(
            CheckId::from_str_name("blocking-sleep-in-async"),
            Some(CheckId::BlockingSleepInAsync)
        );
        assert_eq!(CheckId::from_str_name("unknown-check"), None);
    }
}
