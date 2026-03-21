use serde::Serialize;

use crate::diagnostics::CheckId;

const SCHEMA_VERSION: u32 = 1;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
struct ExplainCatalogEntry {
    id: CheckId,
    detects: &'static [&'static str],
    does_not_detect: &'static [&'static str],
    suggested_fixes: &'static [&'static str],
    references: &'static [ExplainReference],
}

const BLOCKING_SLEEP_DETECTS: &[&str] = &[
    "Direct `std::thread::sleep(...)` calls inside `async fn`, async impl methods, nested `async { ... }` blocks, and async closures.",
    "Module aliases of `std::thread`, including renamed imports such as `use std::thread as blocking_thread; blocking_thread::sleep(...)`.",
    "The exact path expression used at the call site, without broader type or trait resolution.",
];

const BLOCKING_SLEEP_DOES_NOT_DETECT: &[&str] = &[
    "Function imports such as `use std::thread::sleep; sleep(...)`.",
    "Wildcard imports, re-exports, macro-expanded paths, or `use` items declared inside blocks.",
    "Other blocking timer APIs or non-std lookalikes that only happen to be named `sleep`.",
];

const BLOCKING_SLEEP_FIXES: &[&str] = &[
    "Prefer your runtime's async timer, such as `tokio::time::sleep(...).await` for Tokio code.",
    "If the operation must block a thread, move it behind `tokio::task::spawn_blocking` or another clearly synchronous boundary.",
];

const BLOCKING_SLEEP_REFERENCES: &[ExplainReference] = &[
    ExplainReference {
        label: "Tokio `sleep`",
        url: "https://docs.rs/tokio/latest/tokio/time/fn.sleep.html",
    },
    ExplainReference {
        label: "Rust `std::thread::sleep`",
        url: "https://doc.rust-lang.org/std/thread/fn.sleep.html",
    },
];

const BLOCKING_STD_API_DETECTS: &[&str] = &[
    "Direct `std::fs::...` calls inside async contexts for this shipped allowlist: `canonicalize`, `copy`, `create_dir`, `create_dir_all`, `metadata`, `read`, `read_dir`, `read_link`, `read_to_string`, `remove_dir`, `remove_dir_all`, `remove_file`, `rename`, `symlink_metadata`, and `write`.",
    "Module aliases of `std::fs`, including renamed imports such as `use std::fs as blocking_fs; blocking_fs::read_to_string(...)`.",
    "The exact allowlisted function names only; Phase 2 intentionally stays narrow instead of guessing across the whole standard library.",
];

const BLOCKING_STD_API_DOES_NOT_DETECT: &[&str] = &[
    "Function imports such as `use std::fs::read_to_string; read_to_string(...)`.",
    "Other blocking std APIs like `File::open`, wildcard imports, re-exports, macro-expanded paths, or block-local `use` items.",
    "Non-std filesystem crates or local helper modules that look similar to `std::fs`.",
];

const BLOCKING_STD_API_FIXES: &[&str] = &[
    "Prefer `tokio::fs` when the surrounding code is already async and you are on Tokio.",
    "If you must keep the synchronous filesystem call, isolate it behind `tokio::task::spawn_blocking` so the executor thread is not parked on disk I/O.",
];

const BLOCKING_STD_API_REFERENCES: &[ExplainReference] = &[
    ExplainReference {
        label: "Tokio `fs` module",
        url: "https://docs.rs/tokio/latest/tokio/fs/",
    },
    ExplainReference {
        label: "Tokio `spawn_blocking`",
        url: "https://docs.rs/tokio/latest/tokio/task/fn.spawn_blocking.html",
    },
    ExplainReference {
        label: "Rust `std::fs` module",
        url: "https://doc.rust-lang.org/std/fs/index.html",
    },
];

const SYNC_ASYNC_BRIDGE_DETECTS: &[&str] = &[
    "`tokio::runtime::Handle::current().block_on(...)` inside async contexts.",
    "Imported or renamed Tokio runtime types such as `use tokio::runtime::Handle as TokioHandle; TokioHandle::current().block_on(...)` and the same pattern for `Runtime::new().block_on(...)`.",
    "Simple receiver wrappers that the syntax pass strips before matching, including `Runtime::new().unwrap().block_on(...)`, `expect(...)`, `clone()`, and references to the receiver.",
];

const SYNC_ASYNC_BRIDGE_DOES_NOT_DETECT: &[&str] = &[
    "Stored handle/runtime variables such as `let handle = Handle::current(); handle.block_on(...)`.",
    "Other bridge patterns like `Handle::try_current()`, builder-created runtimes, or non-Tokio runtimes.",
    "Macro-expanded paths, wildcard imports, re-exports, or block-local `use` items that would require deeper name resolution.",
];

const SYNC_ASYNC_BRIDGE_FIXES: &[&str] = &[
    "Keep the call chain async instead of re-entering the runtime with `block_on` from async code.",
    "If a synchronous boundary is truly required, move that bridge to a clearly synchronous entry point rather than nesting it inside an async task.",
];

const SYNC_ASYNC_BRIDGE_REFERENCES: &[ExplainReference] = &[
    ExplainReference {
        label: "Tokio `Handle::block_on`",
        url: "https://docs.rs/tokio/latest/tokio/runtime/struct.Handle.html#method.block_on",
    },
    ExplainReference {
        label: "Tokio `Runtime::block_on`",
        url: "https://docs.rs/tokio/latest/tokio/runtime/struct.Runtime.html#method.block_on",
    },
    ExplainReference {
        label: "Tokio `Runtime::new`",
        url: "https://docs.rs/tokio/latest/tokio/runtime/struct.Runtime.html#method.new",
    },
];

const CATALOG: &[ExplainCatalogEntry] = &[
    ExplainCatalogEntry {
        id: CheckId::BlockingSleepInAsync,
        detects: BLOCKING_SLEEP_DETECTS,
        does_not_detect: BLOCKING_SLEEP_DOES_NOT_DETECT,
        suggested_fixes: BLOCKING_SLEEP_FIXES,
        references: BLOCKING_SLEEP_REFERENCES,
    },
    ExplainCatalogEntry {
        id: CheckId::BlockingStdApiInAsync,
        detects: BLOCKING_STD_API_DETECTS,
        does_not_detect: BLOCKING_STD_API_DOES_NOT_DETECT,
        suggested_fixes: BLOCKING_STD_API_FIXES,
        references: BLOCKING_STD_API_REFERENCES,
    },
    ExplainCatalogEntry {
        id: CheckId::SyncAsyncBridgeHazard,
        detects: SYNC_ASYNC_BRIDGE_DETECTS,
        does_not_detect: SYNC_ASYNC_BRIDGE_DOES_NOT_DETECT,
        suggested_fixes: SYNC_ASYNC_BRIDGE_FIXES,
        references: SYNC_ASYNC_BRIDGE_REFERENCES,
    },
];

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ExplainReference {
    pub label: &'static str,
    pub url: &'static str,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
pub struct Explanation {
    pub id: CheckId,
    pub title: &'static str,
    pub summary: &'static str,
    pub detects: Vec<&'static str>,
    pub does_not_detect: Vec<&'static str>,
    pub suggested_fixes: Vec<&'static str>,
    pub references: Vec<ExplainReference>,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
pub struct ExplainReport {
    pub schema_version: u32,
    pub requested_check_id: String,
    pub found: bool,
    pub error: Option<&'static str>,
    pub known_check_ids: Vec<CheckId>,
    pub explanation: Option<Explanation>,
}

pub fn explain(check_id: &str) -> ExplainReport {
    let known_check_ids = CheckId::PHASE_TWO_SHIPPED.to_vec();

    match CATALOG.iter().find(|entry| entry.id.as_str() == check_id) {
        Some(entry) => ExplainReport {
            schema_version: SCHEMA_VERSION,
            requested_check_id: check_id.to_string(),
            found: true,
            error: None,
            known_check_ids,
            explanation: Some(Explanation {
                id: entry.id,
                title: entry.id.title(),
                summary: entry.id.explanation(),
                detects: entry.detects.to_vec(),
                does_not_detect: entry.does_not_detect.to_vec(),
                suggested_fixes: entry.suggested_fixes.to_vec(),
                references: entry.references.to_vec(),
            }),
        },
        None => ExplainReport {
            schema_version: SCHEMA_VERSION,
            requested_check_id: check_id.to_string(),
            found: false,
            error: Some("unknown-check-id"),
            known_check_ids,
            explanation: None,
        },
    }
}

#[cfg(test)]
mod tests {
    use crate::diagnostics::CheckId;

    use super::explain;

    #[test]
    fn known_phase_two_check_returns_canonical_explain_content() {
        let report = explain("blocking-sleep-in-async");

        assert!(report.found);
        assert_eq!(report.error, None);
        assert_eq!(
            report
                .explanation
                .as_ref()
                .map(|explanation| explanation.id),
            Some(CheckId::BlockingSleepInAsync)
        );
        assert!(report
            .explanation
            .as_ref()
            .is_some_and(|explanation| explanation
                .references
                .iter()
                .any(|reference| reference.url
                    == "https://docs.rs/tokio/latest/tokio/time/fn.sleep.html")));
    }

    #[test]
    fn unknown_check_id_reports_known_public_targets() {
        let report = explain("not-a-real-check");

        assert!(!report.found);
        assert_eq!(report.error, Some("unknown-check-id"));
        assert!(report.explanation.is_none());
        assert_eq!(report.known_check_ids, CheckId::PHASE_TWO_SHIPPED.to_vec());
    }

    #[test]
    fn reserved_unshipped_check_is_treated_as_unknown_for_now() {
        let report = explain("guard-across-await");

        assert!(!report.found);
        assert_eq!(report.error, Some("unknown-check-id"));
        assert!(report.explanation.is_none());
    }

    #[test]
    fn explain_catalog_covers_every_shipped_phase_two_check() {
        for id in CheckId::PHASE_TWO_SHIPPED {
            let report = explain(id.as_str());
            assert!(report.found, "missing explain content for {id}");
        }
    }
}
