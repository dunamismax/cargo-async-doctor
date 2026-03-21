use std::str::FromStr;

use serde::Serialize;

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
                "std::thread::sleep parks the OS thread, so calling it in async code blocks unrelated work scheduled on the same executor thread."
            }
            Self::BlockingStdApiInAsync => {
                "Synchronous std::fs calls block the executor thread until the filesystem operation finishes."
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
                "Prefer the runtime's async timer, such as tokio::time::sleep(...).await, or move the blocking sleep off the async path.",
            ),
            Self::BlockingStdApiInAsync => Some(
                "Prefer tokio::fs for async filesystem work or isolate the blocking std::fs call behind tokio::task::spawn_blocking.",
            ),
            Self::SyncAsyncBridgeHazard => Some(
                "Keep the call chain async instead of blocking, or move the sync/async bridge to a clearly synchronous entry point.",
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
pub struct Diagnostic {
    pub id: CheckId,
    pub severity: Severity,
    pub message: String,
    pub help: Option<String>,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
pub struct ScanTarget {
    pub workspace: bool,
    pub manifest_path: Option<String>,
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
    pub placeholder: bool,
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
