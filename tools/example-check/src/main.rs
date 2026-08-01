use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use syn::visit::Visit;

const FRAMEWORK: &str = "crates/framework";

fn main() -> ExitCode {
    if !Path::new(FRAMEWORK).is_dir() {
        eprintln!("example-check: run from the repository root; {FRAMEWORK} not found");
        return ExitCode::FAILURE;
    }

    let mut faults = Vec::new();
    for crate_dir in framework_crate_dirs() {
        faults.extend(faults_in(&crate_dir));
    }

    if faults.is_empty() {
        println!("every example spawns what its crate queries");
        println!("every example reads back what its crate works out");
        println!("no example rebuilds settings the plugin already inserted");
        return ExitCode::SUCCESS;
    }

    for fault in &faults {
        eprintln!("{fault}");
    }
    eprintln!();
    eprintln!("an example must never stand in for the crate it shows");
    ExitCode::FAILURE
}

fn faults_in(crate_dir: &Path) -> Vec<String> {
    let name = crate_dir
        .file_name()
        .map(|leaf| leaf.to_string_lossy().into_owned())
        .unwrap_or_default();

    let examples = example_dirs(crate_dir);
    if examples.is_empty() {
        return Vec::new();
    }

    let mut source = Reading::default();
    let mut sources = Vec::new();
    collect_rs(&crate_dir.join("src"), &mut sources);
    for path in &sources {
        source.read(path);
    }

    let queried: BTreeSet<String> = source
        .queried
        .intersection(&source.declared)
        .cloned()
        .collect();
    let written: BTreeSet<String> = source
        .written
        .intersection(&source.declared)
        .cloned()
        .collect();
    let inserted: BTreeSet<String> = source
        .inserted
        .iter()
        .filter(|kind| kind.ends_with("Settings") || kind.ends_with("Style"))
        .cloned()
        .collect();

    let mut faults = Vec::new();
    for example in examples {
        let label = example
            .file_name()
            .map(|leaf| leaf.to_string_lossy().into_owned())
            .unwrap_or_default();

        let mut used = Reading::default();
        let mut files = Vec::new();
        collect_rs(&example, &mut files);
        for path in &files {
            used.read(path);
        }

        for kind in &queried {
            if !used.placed.contains(kind) {
                faults.push(format!(
                    "{name} example {label}: the crate queries {kind} and the example never sets it, so that system never runs"
                ));
            }
        }

        let named: BTreeSet<String> = used.read_back.union(&used.queried).cloned().collect();
        let idle: BTreeSet<String> = used
            .bound_idle
            .difference(&used.bound_used)
            .cloned()
            .collect();
        let read: BTreeSet<String> = named.difference(&idle).cloned().collect();

        for kind in &written {
            if idle.contains(kind) {
                faults.push(format!(
                    "{name} example {label}: the example queries {kind} and never uses what it bound, so the crate's answer is thrown away"
                ));
            }
        }

        for kind in &written {
            if !read.contains(kind) {
                faults.push(format!(
                    "{name} example {label}: the crate works {kind} out and the example never reads it back"
                ));
            }
        }

        for kind in &inserted {
            if used.built.contains(kind) && !read.contains(kind) {
                faults.push(format!(
                    "{name} example {label}: {kind} is built again beside the one the plugin inserted"
                ));
            }
        }
    }
    faults
}

#[derive(Default)]
struct Reading {
    declared: BTreeSet<String>,
    queried: BTreeSet<String>,
    written: BTreeSet<String>,
    inserted: BTreeSet<String>,
    placed: BTreeSet<String>,
    built: BTreeSet<String>,
    read_back: BTreeSet<String>,
    bound_used: BTreeSet<String>,
    bound_idle: BTreeSet<String>,
    configuring: usize,
}

impl Reading {
    fn read(&mut self, path: &Path) {
        let Ok(text) = fs::read_to_string(path) else {
            return;
        };
        let Ok(file) = syn::parse_file(&text) else {
            return;
        };
        self.visit_file(&file);
    }
}

fn query_elements(kind: &syn::Type) -> Option<Vec<String>> {
    let syn::Type::Path(path) = kind else {
        return None;
    };
    let segment = path.path.segments.last()?;
    if segment.ident != "Query" {
        return None;
    }
    let syn::PathArguments::AngleBracketed(arguments) = &segment.arguments else {
        return None;
    };
    let syn::GenericArgument::Type(inner) = arguments.args.first()? else {
        return None;
    };
    match inner {
        syn::Type::Tuple(tuple) => Some(tuple.elems.iter().filter_map(element_of).collect()),
        other => element_of(other).map(|found| vec![found]),
    }
}

#[derive(Default)]
struct Walks {
    found: Vec<(String, syn::ExprClosure)>,
}

impl<'ast> Visit<'ast> for Walks {
    fn visit_expr_method_call(&mut self, call: &'ast syn::ExprMethodCall) {
        if (call.method == "for_each"
            || call.method == "for_each_mut"
            || call.method == "for_entity")
            && let syn::Expr::Path(receiver) = call.receiver.as_ref()
            && let Some(name) = last_segment(&receiver.path)
            && let Some(closure) = call.args.iter().find_map(|argument| match argument {
                syn::Expr::Closure(closure) => Some(closure),
                _ => None,
            })
        {
            self.found.push((name, closure.clone()));
        }
        syn::visit::visit_expr_method_call(self, call);
    }
}

#[derive(Default)]
struct Names {
    used: BTreeSet<String>,
}

impl<'ast> Visit<'ast> for Names {
    fn visit_expr_path(&mut self, path: &'ast syn::ExprPath) {
        if path.path.segments.len() == 1
            && let Some(name) = last_segment(&path.path)
        {
            self.used.insert(name);
        }
        syn::visit::visit_expr_path(self, path);
    }

    fn visit_macro(&mut self, called: &'ast syn::Macro) {
        self.take_tokens(called.tokens.clone());
        syn::visit::visit_macro(self, called);
    }
}

impl Names {
    fn take_tokens(&mut self, tokens: proc_macro2::TokenStream) {
        for token in tokens {
            match token {
                proc_macro2::TokenTree::Ident(found) => {
                    self.used.insert(found.to_string());
                }
                proc_macro2::TokenTree::Group(group) => self.take_tokens(group.stream()),
                _ => {}
            }
        }
    }
}

fn last_segment(path: &syn::Path) -> Option<String> {
    path.segments
        .last()
        .map(|segment| segment.ident.to_string())
}

fn element_of(kind: &syn::Type) -> Option<String> {
    match kind {
        syn::Type::Reference(reference) => element_of(&reference.elem),
        syn::Type::Path(path) => last_segment(&path.path),
        _ => None,
    }
}

fn is_written(kind: &syn::Type) -> bool {
    match kind {
        syn::Type::Reference(reference) => reference.mutability.is_some(),
        _ => false,
    }
}

impl<'ast> Visit<'ast> for Reading {
    fn visit_item_fn(&mut self, item: &'ast syn::ItemFn) {
        self.take_query_bindings(&item.sig, &item.block);
        syn::visit::visit_item_fn(self, item);
    }

    fn visit_item_struct(&mut self, declared: &'ast syn::ItemStruct) {
        if matches!(declared.vis, syn::Visibility::Public(_)) {
            self.declared.insert(declared.ident.to_string());
        }
        syn::visit::visit_item_struct(self, declared);
    }

    fn visit_type_path(&mut self, path: &'ast syn::TypePath) {
        if let Some(segment) = path.path.segments.last()
            && let syn::PathArguments::AngleBracketed(arguments) = &segment.arguments
        {
            let name = segment.ident.to_string();
            if name == "Query" {
                for argument in &arguments.args {
                    if let syn::GenericArgument::Type(kind) = argument {
                        self.take_query_element(kind);
                    }
                }
            }
            if name == "Res" || name == "ResMut" {
                for argument in &arguments.args {
                    if let syn::GenericArgument::Type(kind) = argument
                        && let Some(found) = element_of(kind)
                    {
                        self.read_back.insert(found);
                    }
                }
            }
        }
        syn::visit::visit_type_path(self, path);
    }

    fn visit_expr_method_call(&mut self, call: &'ast syn::ExprMethodCall) {
        let method = call.method.to_string();
        if method == "add_plugin" || method == "add_plugins" {
            self.configuring += 1;
            syn::visit::visit_expr_method_call(self, call);
            self.configuring -= 1;
            return;
        }
        if method == "set" || method == "insert_resource" {
            if let Some(argument) = call.args.last()
                && let Some(kind) = constructed(argument)
            {
                self.placed.insert(kind.clone());
                if method == "insert_resource" {
                    self.inserted.insert(kind);
                }
            }
            if method == "insert_resource"
                && let Some(syn::Expr::Field(field)) = call.args.last()
                && let syn::Member::Named(named) = &field.member
            {
                self.inserted.insert(named.to_string());
            }
        }
        if method == "spawn" {
            for argument in &call.args {
                for kind in tuple_members(argument) {
                    self.placed.insert(kind);
                }
            }
        }
        if method == "res" || method == "res_mut" || method == "get" || method == "get_mut" {
            for kind in turbofish_types(&call.turbofish) {
                self.read_back.insert(kind);
            }
        }
        if method == "resource_or_insert_default" {
            for kind in turbofish_types(&call.turbofish) {
                self.placed.insert(kind.clone());
                self.inserted.insert(kind);
            }
        }
        syn::visit::visit_expr_method_call(self, call);
    }

    fn visit_expr_struct(&mut self, built: &'ast syn::ExprStruct) {
        if self.configuring == 0
            && let Some(kind) = last_segment(&built.path)
        {
            self.built.insert(kind);
        }
        syn::visit::visit_expr_struct(self, built);
    }

    fn visit_expr_field(&mut self, field: &'ast syn::ExprField) {
        if let syn::Expr::Path(path) = field.base.as_ref()
            && let Some(kind) = last_segment(&path.path)
            && kind
                .chars()
                .next()
                .is_some_and(|letter| letter.is_uppercase())
        {
            self.read_back.insert(kind);
        }
        syn::visit::visit_expr_field(self, field);
    }
}

impl Reading {
    fn take_query_bindings(&mut self, signature: &syn::Signature, body: &syn::Block) {
        let mut queries: Vec<(String, Vec<String>)> = Vec::new();
        for input in &signature.inputs {
            let syn::FnArg::Typed(typed) = input else {
                continue;
            };
            let syn::Pat::Ident(named) = typed.pat.as_ref() else {
                continue;
            };
            let Some(elements) = query_elements(&typed.ty) else {
                continue;
            };
            queries.push((named.ident.to_string(), elements));
        }
        if queries.is_empty() {
            return;
        }

        let mut walks = Walks::default();
        walks.visit_block(body);
        for (receiver, closure) in walks.found {
            let Some((_, elements)) = queries.iter().find(|(name, _)| *name == receiver) else {
                continue;
            };
            let Some(tuple) = closure.inputs.last() else {
                continue;
            };
            let syn::Pat::Tuple(bound) = tuple else {
                continue;
            };
            let mut names = Names::default();
            names.visit_expr(&closure.body);
            for (index, pattern) in bound.elems.iter().enumerate() {
                let Some(kind) = elements.get(index) else {
                    continue;
                };
                match pattern {
                    syn::Pat::Ident(named) => {
                        let bound_name = named.ident.to_string();
                        if bound_name.starts_with('_') || !names.used.contains(&bound_name) {
                            self.bound_idle.insert(kind.clone());
                        } else {
                            self.bound_used.insert(kind.clone());
                        }
                    }
                    syn::Pat::Wild(_) => {
                        self.bound_idle.insert(kind.clone());
                    }
                    _ => {
                        self.bound_used.insert(kind.clone());
                    }
                }
            }
        }
    }

    fn take_query_element(&mut self, kind: &syn::Type) {
        match kind {
            syn::Type::Tuple(tuple) => {
                for member in &tuple.elems {
                    self.take_query_element(member);
                }
            }
            other => {
                if let Some(found) = element_of(other) {
                    self.queried.insert(found.clone());
                    if is_written(other) {
                        self.written.insert(found);
                    }
                }
            }
        }
    }
}

fn constructed(argument: &syn::Expr) -> Option<String> {
    match argument {
        syn::Expr::Call(call) => match call.func.as_ref() {
            syn::Expr::Path(path) => last_segment(&path.path),
            _ => None,
        },
        syn::Expr::Struct(built) => last_segment(&built.path),
        syn::Expr::Path(path) => last_segment(&path.path),
        _ => None,
    }
}

fn tuple_members(argument: &syn::Expr) -> Vec<String> {
    match argument {
        syn::Expr::Tuple(tuple) => tuple.elems.iter().filter_map(constructed).collect(),
        other => constructed(other).into_iter().collect(),
    }
}

fn turbofish_types(turbofish: &Option<syn::AngleBracketedGenericArguments>) -> Vec<String> {
    let Some(arguments) = turbofish else {
        return Vec::new();
    };
    arguments
        .args
        .iter()
        .filter_map(|argument| match argument {
            syn::GenericArgument::Type(kind) => element_of(kind),
            _ => None,
        })
        .collect()
}

fn framework_crate_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    let Ok(groups) = fs::read_dir(FRAMEWORK) else {
        return dirs;
    };
    for group in groups.flatten() {
        let path = group.path();
        if !path.is_dir() {
            continue;
        }
        let Ok(entries) = fs::read_dir(&path) else {
            continue;
        };
        for entry in entries.flatten() {
            if entry.path().join("src").is_dir() {
                dirs.push(entry.path());
            }
        }
    }
    dirs.sort();
    dirs
}

fn example_dirs(crate_dir: &Path) -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    let Ok(entries) = fs::read_dir(crate_dir.join("examples")) else {
        return dirs;
    };
    for entry in entries.flatten() {
        if entry.path().is_dir() {
            dirs.push(entry.path());
        }
    }
    dirs.sort();
    dirs
}

fn collect_rs(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_rs(&path, out);
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            out.push(path);
        }
    }
}
