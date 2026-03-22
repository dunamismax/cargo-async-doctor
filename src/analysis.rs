use std::collections::BTreeSet;
use std::fs;
use std::path::{Component, Path, PathBuf};

use anyhow::{Context, Result};
use proc_macro2::Span;
use syn::spanned::Spanned;
use syn::visit::{self, Visit};
use syn::{
    Expr, ExprAsync, ExprCall, ExprClosure, ExprLit, ExprMethodCall, ImplItem, ImplItemFn, Item,
    ItemFn, ItemMod, Lit, Meta, Path as SynPath, UseTree,
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
    pub target_roots: Vec<PathBuf>,
}

#[derive(Debug, Clone, Copy, Eq, Ord, PartialEq, PartialOrd)]
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

#[derive(Debug, Clone)]
struct ModuleContext<'a> {
    package: &'a PackageContext,
    file: PathBuf,
    child_dir: PathBuf,
}

impl<'a> ModuleContext<'a> {
    fn crate_root(package: &'a PackageContext, file: &Path) -> Self {
        Self {
            package,
            file: normalize_path(file),
            child_dir: file
                .parent()
                .map(normalize_path)
                .unwrap_or_else(|| package.root_dir.clone()),
        }
    }

    fn inline_child(&self, module_name: &str) -> Self {
        Self {
            package: self.package,
            file: self.file.clone(),
            child_dir: normalize_path(&self.child_dir.join(module_name)),
        }
    }

    fn external_child(&self, file: PathBuf) -> Self {
        Self {
            package: self.package,
            child_dir: child_module_dir_for_file(&file),
            file,
        }
    }
}

pub fn analyze_package(package: &PackageContext) -> Result<AnalysisResult> {
    if package.target_roots.is_empty() {
        return Ok(AnalysisResult {
            findings: Vec::new(),
            notes: vec![format!(
                "Package `{}` has no Rust crate targets in Cargo metadata.",
                package.name
            )],
        });
    }

    let mut findings = Vec::new();
    let mut analyzed_roots = BTreeSet::new();

    for target_root in &package.target_roots {
        if !analyzed_roots.insert(target_root.clone()) {
            continue;
        }

        let mut visited_files = BTreeSet::new();
        let module = ModuleContext::crate_root(package, target_root);
        analyze_module_file(&module, &mut visited_files, &mut findings)?;
    }

    findings.sort_by(|left, right| {
        left.package_name
            .cmp(&right.package_name)
            .then_with(|| left.file.cmp(&right.file))
            .then_with(|| left.span.cmp(&right.span))
            .then_with(|| left.id.as_str().cmp(right.id.as_str()))
            .then_with(|| left.matched.cmp(&right.matched))
    });
    findings.dedup();

    Ok(AnalysisResult {
        findings,
        notes: Vec::new(),
    })
}

fn analyze_module_file(
    module: &ModuleContext<'_>,
    visited_files: &mut BTreeSet<PathBuf>,
    findings: &mut Vec<Finding>,
) -> Result<()> {
    if !visited_files.insert(module.file.clone()) {
        return Ok(());
    }

    let source = fs::read_to_string(&module.file).with_context(|| {
        format!(
            "failed to read Rust source file `{}`",
            module.file.display()
        )
    })?;
    let syntax = syn::parse_file(&source).with_context(|| {
        format!(
            "failed to parse Rust source file `{}`",
            module.file.display()
        )
    })?;

    let mut errors = Vec::new();
    analyze_module_items(&syntax.items, module, visited_files, findings, &mut errors);

    if let Some(error) = errors.into_iter().next() {
        return Err(error);
    }

    Ok(())
}

fn analyze_module_items(
    items: &[Item],
    module: &ModuleContext<'_>,
    visited_files: &mut BTreeSet<PathBuf>,
    findings: &mut Vec<Finding>,
    errors: &mut Vec<anyhow::Error>,
) {
    let env = AliasEnv::default().extend_from_items(items);

    for item in items {
        match item {
            Item::Fn(function) => {
                analyze_function(function, &env, module, findings, visited_files, errors)
            }
            Item::Impl(item_impl) => {
                for item in &item_impl.items {
                    if let ImplItem::Fn(function) = item {
                        analyze_impl_function(
                            function,
                            &env,
                            module,
                            findings,
                            visited_files,
                            errors,
                        );
                    }
                }
            }
            Item::Mod(item_mod) => {
                analyze_nested_module(item_mod, module, findings, visited_files, errors);
            }
            _ => {}
        }
    }
}

fn analyze_nested_module(
    item_mod: &ItemMod,
    module: &ModuleContext<'_>,
    findings: &mut Vec<Finding>,
    visited_files: &mut BTreeSet<PathBuf>,
    errors: &mut Vec<anyhow::Error>,
) {
    if let Some((_, items)) = &item_mod.content {
        let child = module.inline_child(&item_mod.ident.to_string());
        analyze_module_items(items, &child, visited_files, findings, errors);
        return;
    }

    let Some(path) = resolve_external_module(module, item_mod) else {
        return;
    };

    let child = module.external_child(path);
    if let Err(error) = analyze_module_file(&child, visited_files, findings) {
        errors.push(error);
    }
}

fn resolve_external_module(module: &ModuleContext<'_>, item_mod: &ItemMod) -> Option<PathBuf> {
    let path_attr = module_path_attribute(item_mod).map(PathBuf::from);

    let candidates = if let Some(path_attr) = path_attr {
        module_path_attribute_candidates(module, &path_attr)
    } else {
        let module_name = item_mod.ident.to_string();
        vec![
            normalize_path(&module.child_dir.join(format!("{module_name}.rs"))),
            normalize_path(&module.child_dir.join(&module_name).join("mod.rs")),
        ]
    };

    candidates.into_iter().find(|candidate| candidate.exists())
}

fn module_path_attribute(item_mod: &ItemMod) -> Option<String> {
    item_mod.attrs.iter().find_map(|attribute| {
        if !attribute.path().is_ident("path") {
            return None;
        }

        let Meta::NameValue(meta) = &attribute.meta else {
            return None;
        };
        let Expr::Lit(ExprLit {
            lit: Lit::Str(path),
            ..
        }) = &meta.value
        else {
            return None;
        };

        Some(path.value())
    })
}

fn module_path_attribute_candidates(module: &ModuleContext<'_>, path_attr: &Path) -> Vec<PathBuf> {
    if path_attr.is_absolute() {
        return vec![normalize_path(path_attr)];
    }

    let mut candidates = Vec::new();

    if let Some(file_parent) = module.file.parent() {
        candidates.push(normalize_path(&file_parent.join(path_attr)));
    }

    let child_dir_candidate = normalize_path(&module.child_dir.join(path_attr));
    if !candidates.contains(&child_dir_candidate) {
        candidates.push(child_dir_candidate);
    }

    candidates
}

fn child_module_dir_for_file(file: &Path) -> PathBuf {
    let file = normalize_path(file);
    let Some(parent) = file.parent() else {
        return PathBuf::new();
    };

    match file.file_stem().and_then(|stem| stem.to_str()) {
        Some("mod") => parent.to_path_buf(),
        Some(stem) => parent.join(stem),
        None => parent.to_path_buf(),
    }
}

fn analyze_function(
    function: &ItemFn,
    env: &AliasEnv,
    module: &ModuleContext<'_>,
    findings: &mut Vec<Finding>,
    visited_files: &mut BTreeSet<PathBuf>,
    errors: &mut Vec<anyhow::Error>,
) {
    let mut visitor = AsyncContextVisitor::new(
        module,
        env.clone(),
        findings,
        visited_files,
        errors,
        usize::from(function.sig.asyncness.is_some()),
    );
    visitor.visit_block(&function.block);
}

fn analyze_impl_function(
    function: &ImplItemFn,
    env: &AliasEnv,
    module: &ModuleContext<'_>,
    findings: &mut Vec<Finding>,
    visited_files: &mut BTreeSet<PathBuf>,
    errors: &mut Vec<anyhow::Error>,
) {
    let mut visitor = AsyncContextVisitor::new(
        module,
        env.clone(),
        findings,
        visited_files,
        errors,
        usize::from(function.sig.asyncness.is_some()),
    );
    visitor.visit_block(&function.block);
}

struct AsyncContextVisitor<'a> {
    module: ModuleContext<'a>,
    env: AliasEnv,
    findings: &'a mut Vec<Finding>,
    visited_files: &'a mut BTreeSet<PathBuf>,
    errors: &'a mut Vec<anyhow::Error>,
    async_depth: usize,
}

impl<'a> AsyncContextVisitor<'a> {
    fn new(
        module: &ModuleContext<'a>,
        env: AliasEnv,
        findings: &'a mut Vec<Finding>,
        visited_files: &'a mut BTreeSet<PathBuf>,
        errors: &'a mut Vec<anyhow::Error>,
        async_depth: usize,
    ) -> Self {
        Self {
            module: module.clone(),
            env,
            findings,
            visited_files,
            errors,
            async_depth,
        }
    }

    fn in_async_context(&self) -> bool {
        self.async_depth > 0
    }

    fn push_finding(&mut self, id: CheckId, matched: String, span: Option<SourceSpan>) {
        self.findings.push(Finding {
            id,
            package_name: self.module.package.name.clone(),
            package_manifest_path: self.module.package.manifest_path.clone(),
            package_root: self.module.package.root_dir.clone(),
            workspace_root: self.module.package.workspace_root.clone(),
            file: self.module.file.clone(),
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
            &self.module,
            self.env.clone(),
            self.findings,
            self.visited_files,
            self.errors,
            usize::from(node.asyncness.is_some()),
        );
        nested.visit_expr(&node.body);
    }

    fn visit_item_fn(&mut self, node: &ItemFn) {
        let mut nested = AsyncContextVisitor::new(
            &self.module,
            self.env.clone(),
            self.findings,
            self.visited_files,
            self.errors,
            usize::from(node.sig.asyncness.is_some()),
        );
        nested.visit_block(&node.block);
    }

    fn visit_item_mod(&mut self, node: &ItemMod) {
        analyze_nested_module(
            node,
            &self.module,
            self.findings,
            self.visited_files,
            self.errors,
        );
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

fn normalize_path(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();

    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                normalized.pop();
            }
            Component::RootDir | Component::Prefix(_) | Component::Normal(_) => {
                normalized.push(component.as_os_str());
            }
        }
    }

    normalized
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use crate::diagnostics::CheckId;

    use super::PackageContext;

    fn package_context() -> PackageContext {
        PackageContext {
            name: "fixture-package".to_string(),
            manifest_path: PathBuf::from("Cargo.toml"),
            root_dir: PathBuf::from("."),
            workspace_root: PathBuf::from("."),
            target_roots: vec![PathBuf::from("src/main.rs")],
        }
    }

    fn analyze_syntax(
        package: &PackageContext,
        path: &Path,
        syntax: &syn::File,
    ) -> Vec<super::Finding> {
        let module = super::ModuleContext::crate_root(package, path);
        let mut findings = Vec::new();
        let mut visited_files = std::collections::BTreeSet::new();
        let mut errors = Vec::new();

        super::analyze_module_items(
            &syntax.items,
            &module,
            &mut visited_files,
            &mut findings,
            &mut errors,
        );

        assert!(errors.is_empty(), "unexpected analysis errors: {errors:?}");
        findings
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
    fn nested_inline_modules_do_not_inherit_parent_aliases() {
        let syntax = syn::parse_file(
            r#"
            use std::{fs, thread};
            use tokio::runtime::Handle;

            mod nested {
                async fn quiet() {
                    thread::sleep(std::time::Duration::from_millis(1));
                    fs::read_to_string("Cargo.toml");
                    Handle::current().block_on(async {});
                }
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
