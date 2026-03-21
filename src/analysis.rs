use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use proc_macro2::Span;
use syn::spanned::Spanned;
use syn::visit::{self, Visit};
use syn::{
    Expr, ExprAsync, ExprCall, ExprClosure, ExprMethodCall, File, ImplItem, ImplItemFn, Item,
    ItemFn, ItemMod, Path as SynPath, UseTree,
};

use crate::diagnostics::CheckId;

const BLOCKING_STD_FS_FUNCTIONS: &[&str] = &[
    "canonicalize",
    "copy",
    "create_dir",
    "create_dir_all",
    "metadata",
    "read",
    "read_dir",
    "read_link",
    "read_to_string",
    "remove_dir",
    "remove_dir_all",
    "remove_file",
    "rename",
    "symlink_metadata",
    "write",
];

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct PackageContext {
    pub name: String,
    pub manifest_path: PathBuf,
    pub root_dir: PathBuf,
    pub workspace_root: PathBuf,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct SourceSpan {
    pub start_line: usize,
    pub start_column: usize,
    pub end_line: usize,
    pub end_column: usize,
}

impl SourceSpan {
    fn from_syn_span(span: Span) -> Option<Self> {
        let start = span.start();
        let end = span.end();

        if start.line == 0 || end.line == 0 {
            return None;
        }

        Some(Self {
            start_line: start.line,
            start_column: start.column + 1,
            end_line: end.line,
            end_column: end.column + 1,
        })
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Finding {
    pub id: CheckId,
    pub package_name: String,
    pub package_manifest_path: PathBuf,
    pub package_root: PathBuf,
    pub workspace_root: PathBuf,
    pub file: PathBuf,
    pub matched: String,
    pub span: Option<SourceSpan>,
}

#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub struct AnalysisResult {
    pub findings: Vec<Finding>,
    pub notes: Vec<String>,
}

#[derive(Debug, Default, Clone)]
struct AliasEnv {
    std_thread_modules: BTreeSet<String>,
    std_fs_modules: BTreeSet<String>,
    tokio_handle_types: BTreeSet<String>,
    tokio_runtime_types: BTreeSet<String>,
}

impl AliasEnv {
    fn extend_from_items(&self, items: &[Item]) -> Self {
        let mut env = self.clone();

        for item in items {
            if let Item::Use(item_use) = item {
                env.record_use_tree(&item_use.tree, &[]);
            }
        }

        env
    }

    fn record_use_tree(&mut self, tree: &UseTree, prefix: &[String]) {
        match tree {
            UseTree::Path(path) => {
                let mut next = prefix.to_vec();
                next.push(path.ident.to_string());
                self.record_use_tree(&path.tree, &next);
            }
            UseTree::Name(name) => {
                if name.ident == "self" {
                    if let Some(last) = prefix.last() {
                        self.record_import(prefix, last.clone());
                    }
                } else {
                    let mut full = prefix.to_vec();
                    full.push(name.ident.to_string());
                    self.record_import(&full, name.ident.to_string());
                }
            }
            UseTree::Rename(rename) => {
                if rename.ident == "self" {
                    self.record_import(prefix, rename.rename.to_string());
                } else {
                    let mut full = prefix.to_vec();
                    full.push(rename.ident.to_string());
                    self.record_import(&full, rename.rename.to_string());
                }
            }
            UseTree::Group(group) => {
                for item in &group.items {
                    self.record_use_tree(item, prefix);
                }
            }
            UseTree::Glob(_) => {}
        }
    }

    fn record_import(&mut self, full_path: &[String], local_name: String) {
        let full_path: Vec<&str> = full_path.iter().map(String::as_str).collect();

        match full_path.as_slice() {
            ["std", "thread"] => {
                self.std_thread_modules.insert(local_name);
            }
            ["std", "fs"] => {
                self.std_fs_modules.insert(local_name);
            }
            ["tokio", "runtime", "Handle"] => {
                self.tokio_handle_types.insert(local_name);
            }
            ["tokio", "runtime", "Runtime"] => {
                self.tokio_runtime_types.insert(local_name);
            }
            _ => {}
        }
    }
}

pub fn analyze_package(package: &PackageContext) -> Result<AnalysisResult> {
    let source_dir = package.root_dir.join("src");

    if !source_dir.exists() {
        return Ok(AnalysisResult {
            findings: Vec::new(),
            notes: vec![format!(
                "Package `{}` has no `src/` directory under `{}`.",
                package.name,
                package.root_dir.display()
            )],
        });
    }

    let mut findings = Vec::new();

    for file in rust_files_under(&source_dir)? {
        findings.extend(analyze_file(package, &file)?);
    }

    findings.sort_by(|left, right| {
        left.package_name
            .cmp(&right.package_name)
            .then_with(|| left.file.cmp(&right.file))
            .then_with(|| left.id.as_str().cmp(right.id.as_str()))
            .then_with(|| left.matched.cmp(&right.matched))
    });

    Ok(AnalysisResult {
        findings,
        notes: Vec::new(),
    })
}

fn analyze_file(package: &PackageContext, path: &Path) -> Result<Vec<Finding>> {
    let source = fs::read_to_string(path)
        .with_context(|| format!("failed to read Rust source file `{}`", path.display()))?;
    let syntax = syn::parse_file(&source)
        .with_context(|| format!("failed to parse Rust source file `{}`", path.display()))?;
    Ok(analyze_syntax(package, path, &syntax))
}

fn analyze_syntax(package: &PackageContext, path: &Path, syntax: &File) -> Vec<Finding> {
    let mut findings = Vec::new();
    analyze_items(
        &syntax.items,
        &AliasEnv::default(),
        package,
        path,
        &mut findings,
    );
    findings
}

fn analyze_items(
    items: &[Item],
    parent_env: &AliasEnv,
    package: &PackageContext,
    file: &Path,
    findings: &mut Vec<Finding>,
) {
    let env = parent_env.extend_from_items(items);

    for item in items {
        match item {
            Item::Fn(function) => analyze_function(function, &env, package, file, findings),
            Item::Impl(item_impl) => {
                for item in &item_impl.items {
                    if let ImplItem::Fn(function) = item {
                        analyze_impl_function(function, &env, package, file, findings);
                    }
                }
            }
            Item::Mod(item_mod) => {
                if let Some((_, items)) = &item_mod.content {
                    analyze_items(items, &env, package, file, findings);
                }
            }
            _ => {}
        }
    }
}

fn analyze_function(
    function: &ItemFn,
    env: &AliasEnv,
    package: &PackageContext,
    file: &Path,
    findings: &mut Vec<Finding>,
) {
    let mut visitor = AsyncContextVisitor::new(
        package,
        file,
        env.clone(),
        findings,
        usize::from(function.sig.asyncness.is_some()),
    );
    visitor.visit_block(&function.block);
}

fn analyze_impl_function(
    function: &ImplItemFn,
    env: &AliasEnv,
    package: &PackageContext,
    file: &Path,
    findings: &mut Vec<Finding>,
) {
    let mut visitor = AsyncContextVisitor::new(
        package,
        file,
        env.clone(),
        findings,
        usize::from(function.sig.asyncness.is_some()),
    );
    visitor.visit_block(&function.block);
}

fn rust_files_under(dir: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    visit_dir(dir, &mut files)?;
    files.sort();
    Ok(files)
}

fn visit_dir(dir: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
    let mut entries: Vec<_> = fs::read_dir(dir)
        .with_context(|| format!("failed to read directory `{}`", dir.display()))?
        .collect::<std::result::Result<Vec<_>, _>>()
        .with_context(|| format!("failed to iterate directory `{}`", dir.display()))?;
    entries.sort_by_key(|entry| entry.path());

    for entry in entries {
        let path = entry.path();
        if path.is_dir() {
            visit_dir(&path, files)?;
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            files.push(path);
        }
    }

    Ok(())
}

struct AsyncContextVisitor<'a> {
    package: &'a PackageContext,
    file: &'a Path,
    env: AliasEnv,
    findings: &'a mut Vec<Finding>,
    async_depth: usize,
}

impl<'a> AsyncContextVisitor<'a> {
    fn new(
        package: &'a PackageContext,
        file: &'a Path,
        env: AliasEnv,
        findings: &'a mut Vec<Finding>,
        async_depth: usize,
    ) -> Self {
        Self {
            package,
            file,
            env,
            findings,
            async_depth,
        }
    }

    fn in_async_context(&self) -> bool {
        self.async_depth > 0
    }

    fn push_finding(&mut self, id: CheckId, matched: String, span: Option<SourceSpan>) {
        self.findings.push(Finding {
            id,
            package_name: self.package.name.clone(),
            package_manifest_path: self.package.manifest_path.clone(),
            package_root: self.package.root_dir.clone(),
            workspace_root: self.package.workspace_root.clone(),
            file: self.file.to_path_buf(),
            matched,
            span,
        });
    }
}

impl Visit<'_> for AsyncContextVisitor<'_> {
    fn visit_expr_async(&mut self, node: &ExprAsync) {
        self.async_depth += 1;
        visit::visit_block(self, &node.block);
        self.async_depth -= 1;
    }

    fn visit_expr_closure(&mut self, node: &ExprClosure) {
        let mut nested = AsyncContextVisitor::new(
            self.package,
            self.file,
            self.env.clone(),
            self.findings,
            usize::from(node.asyncness.is_some()),
        );
        nested.visit_expr(&node.body);
    }

    fn visit_item_fn(&mut self, node: &ItemFn) {
        let mut nested = AsyncContextVisitor::new(
            self.package,
            self.file,
            self.env.clone(),
            self.findings,
            usize::from(node.sig.asyncness.is_some()),
        );
        nested.visit_block(&node.block);
    }

    fn visit_item_mod(&mut self, node: &ItemMod) {
        if let Some((_, items)) = &node.content {
            analyze_items(items, &self.env, self.package, self.file, self.findings);
        }
    }

    fn visit_expr_call(&mut self, node: &ExprCall) {
        if self.in_async_context() {
            if let Some(matched) = blocking_sleep_match(&node.func, &self.env) {
                self.push_finding(
                    CheckId::BlockingSleepInAsync,
                    matched,
                    SourceSpan::from_syn_span(node.span()),
                );
            } else if let Some(matched) = blocking_std_fs_match(&node.func, &self.env) {
                self.push_finding(
                    CheckId::BlockingStdApiInAsync,
                    matched,
                    SourceSpan::from_syn_span(node.span()),
                );
            }
        }

        visit::visit_expr_call(self, node);
    }

    fn visit_expr_method_call(&mut self, node: &ExprMethodCall) {
        if self.in_async_context() && node.method == "block_on" {
            if let Some(matched) = sync_async_bridge_match(node, &self.env) {
                self.push_finding(
                    CheckId::SyncAsyncBridgeHazard,
                    matched,
                    SourceSpan::from_syn_span(node.span()),
                );
            }
        }

        visit::visit_expr_method_call(self, node);
    }
}

fn blocking_sleep_match(function: &Expr, env: &AliasEnv) -> Option<String> {
    let path = expr_path(function)?;
    let segments = path_segments(path);

    if segments == ["std", "thread", "sleep"] {
        return Some(render_path(path));
    }

    if segments.len() == 2
        && segments[1] == "sleep"
        && env.std_thread_modules.contains(&segments[0])
    {
        return Some(render_path(path));
    }

    None
}

fn blocking_std_fs_match(function: &Expr, env: &AliasEnv) -> Option<String> {
    let path = expr_path(function)?;
    let segments = path_segments(path);

    if segments.len() == 3
        && segments[0] == "std"
        && segments[1] == "fs"
        && BLOCKING_STD_FS_FUNCTIONS.contains(&segments[2].as_str())
    {
        return Some(render_path(path));
    }

    if segments.len() == 2
        && env.std_fs_modules.contains(&segments[0])
        && BLOCKING_STD_FS_FUNCTIONS.contains(&segments[1].as_str())
    {
        return Some(render_path(path));
    }

    None
}

fn sync_async_bridge_match(node: &ExprMethodCall, env: &AliasEnv) -> Option<String> {
    if !matches!(node.method.to_string().as_str(), "block_on") {
        return None;
    }

    let receiver = strip_receiver_wrappers(&node.receiver);

    if is_handle_current(receiver, env) {
        return Some("Handle::current().block_on".to_string());
    }

    if is_runtime_new(receiver, env) {
        return Some("Runtime::new().block_on".to_string());
    }

    None
}

fn is_handle_current(expr: &Expr, env: &AliasEnv) -> bool {
    let Expr::Call(call) = expr else {
        return false;
    };

    let Some(path) = expr_path(&call.func) else {
        return false;
    };
    let segments = path_segments(path);

    segments == ["tokio", "runtime", "Handle", "current"]
        || (segments.len() == 2
            && segments[1] == "current"
            && env.tokio_handle_types.contains(&segments[0]))
}

fn is_runtime_new(expr: &Expr, env: &AliasEnv) -> bool {
    let Expr::Call(call) = expr else {
        return false;
    };

    let Some(path) = expr_path(&call.func) else {
        return false;
    };
    let segments = path_segments(path);

    segments == ["tokio", "runtime", "Runtime", "new"]
        || (segments.len() == 2
            && segments[1] == "new"
            && env.tokio_runtime_types.contains(&segments[0]))
}

fn strip_receiver_wrappers(mut expr: &Expr) -> &Expr {
    loop {
        match expr {
            Expr::MethodCall(method)
                if matches!(
                    method.method.to_string().as_str(),
                    "unwrap" | "expect" | "clone"
                ) =>
            {
                expr = &method.receiver;
            }
            Expr::Reference(reference) => {
                expr = &reference.expr;
            }
            _ => return expr,
        }
    }
}

fn expr_path(expr: &Expr) -> Option<&SynPath> {
    match expr {
        Expr::Path(path) => Some(&path.path),
        _ => None,
    }
}

fn path_segments(path: &SynPath) -> Vec<String> {
    path.segments
        .iter()
        .map(|segment| segment.ident.to_string())
        .collect()
}

fn render_path(path: &SynPath) -> String {
    path.segments
        .iter()
        .map(|segment| segment.ident.to_string())
        .collect::<Vec<_>>()
        .join("::")
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use crate::diagnostics::CheckId;

    use super::{analyze_syntax, PackageContext};

    fn package_context() -> PackageContext {
        PackageContext {
            name: "fixture-package".to_string(),
            manifest_path: PathBuf::from("Cargo.toml"),
            root_dir: PathBuf::from("."),
            workspace_root: PathBuf::from("."),
        }
    }

    #[test]
    fn reports_direct_paths_and_imported_aliases_inside_async_contexts() {
        let syntax = syn::parse_file(
            r#"
            use std::{fs, thread};
            use tokio::runtime::Handle;

            async fn demo() {
                thread::sleep(std::time::Duration::from_millis(1));
                fs::read_to_string("Cargo.toml");
                Handle::current().block_on(async {});
            }
            "#,
        )
        .unwrap();

        let findings = analyze_syntax(&package_context(), Path::new("src/main.rs"), &syntax);
        let ids: Vec<CheckId> = findings.iter().map(|finding| finding.id).collect();

        assert_eq!(
            ids,
            vec![
                CheckId::BlockingSleepInAsync,
                CheckId::BlockingStdApiInAsync,
                CheckId::SyncAsyncBridgeHazard,
            ]
        );
    }

    #[test]
    fn ignores_sync_contexts_and_local_lookalikes() {
        let syntax = syn::parse_file(
            r#"
            mod fs {
                pub fn read_to_string(_: &str) {}
            }

            struct Handle;

            impl Handle {
                fn current() -> Self {
                    Self
                }

                fn block_on<F>(&self, _: F) {}
            }

            fn sync_case() {
                std::thread::sleep(std::time::Duration::from_millis(1));
                std::fs::read_to_string("Cargo.toml");
            }

            async fn async_case() {
                fs::read_to_string("Cargo.toml");
                Handle::current().block_on(async {});
            }
            "#,
        )
        .unwrap();

        let findings = analyze_syntax(&package_context(), Path::new("src/main.rs"), &syntax);

        assert!(findings.is_empty());
    }

    #[test]
    fn captures_line_and_column_information_for_findings() {
        let syntax = syn::parse_file(
            r#"
            async fn demo() {
                std::thread::sleep(std::time::Duration::from_millis(1));
            }
            "#,
        )
        .unwrap();

        let findings = analyze_syntax(&package_context(), Path::new("src/main.rs"), &syntax);
        let span = findings[0].span.expect("expected source span");

        assert_eq!(span.start_line, 3);
        assert!(span.start_column > 1);
        assert_eq!(findings[0].package_name, "fixture-package");
    }
}
