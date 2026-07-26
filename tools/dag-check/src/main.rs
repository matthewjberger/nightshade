//! Fails if the nightshade engine layer has a module dependency cycle.
//!
//! The engine's `crate::ecs` domains, plugins, app, and runner are meant to
//! form a dependency DAG: the substrate is depended on, never depends up.
//! This parses the `crate::` references between modules and runs a
//! strongly-connected-components pass; any component of more than one module
//! is a cycle. Run through `just lint-dag`. Exits non-zero and prints the
//! offending edges when a cycle is found.
//!
//! The engine's top-level modules are discovered from the source tree rather
//! than hardcoded, so a new `crate::something` root module is covered without
//! touching this tool. `render`, `ui`, and `platform` are re-exports of
//! sibling crates (structurally unable to depend back into nightshade), so
//! they are not source modules and are correctly ignored as leaves.

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

const SRC: &str = "crates/nightshade/src";

fn main() -> ExitCode {
    let src = Path::new(SRC);
    if !src.is_dir() {
        eprintln!("dag-check: run from the repository root; {SRC} not found");
        return ExitCode::FAILURE;
    }

    let top_modules = discover_top_modules(src);
    let mut files = Vec::new();
    collect_rs(src, &mut files);

    let mut edges: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let mut origin: HashMap<(String, String), String> = HashMap::new();
    for file in &files {
        let source_module = module_of(file, src);
        let text = strip(&fs::read_to_string(file).unwrap_or_default());
        for reference in crate_refs(&text) {
            let Some(target) = map_target(&reference, &top_modules) else {
                continue;
            };
            if target == source_module {
                continue;
            }
            edges
                .entry(source_module.clone())
                .or_default()
                .insert(target.clone());
            origin
                .entry((source_module.clone(), target.clone()))
                .or_insert_with(|| display_path(file));
        }
    }

    let cycles: Vec<Vec<String>> = strongly_connected(&edges)
        .into_iter()
        .filter(|component| component.len() > 1)
        .collect();

    if cycles.is_empty() {
        println!("engine layer is a DAG (no module cycles)");
        return ExitCode::SUCCESS;
    }

    eprintln!("engine layer has module dependency cycles:");
    for mut component in cycles {
        component.sort();
        let members: BTreeSet<&String> = component.iter().collect();
        eprintln!("  cycle of {}: {:?}", component.len(), component);
        for node in &component {
            let Some(targets) = edges.get(node) else {
                continue;
            };
            for target in targets {
                if members.contains(target) {
                    let where_at = origin
                        .get(&(node.clone(), target.clone()))
                        .map(String::as_str)
                        .unwrap_or("?");
                    eprintln!("    {node} -> {target}   [{where_at}]");
                }
            }
        }
    }
    ExitCode::FAILURE
}

/// The engine's top-level module names, taken from the entries directly under
/// `src` (a `foo.rs` file or a `foo` directory is the module `foo`).
fn discover_top_modules(src: &Path) -> BTreeSet<String> {
    let mut modules = BTreeSet::new();
    let Ok(entries) = fs::read_dir(src) else {
        return modules;
    };
    for entry in entries.flatten() {
        let name = module_name(&entry.path());
        if matches!(name.as_str(), "lib" | "build" | "mod") {
            continue;
        }
        modules.insert(name);
    }
    modules
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

/// The module a source file belongs to: `ecs`, `assets` and `plugins` resolve
/// one level deep (`ecs::camera`), everything else to the top-level module name.
fn module_of(file: &Path, src: &Path) -> String {
    let relative = file.strip_prefix(src).unwrap_or(file);
    let parts: Vec<String> = relative
        .components()
        .map(|component| component.as_os_str().to_string_lossy().into_owned())
        .collect();
    let first = strip_rs(&parts[0]);
    if (first == "ecs" || first == "assets" || first == "plugins") && parts.len() > 1 {
        return format!("{first}::{}", strip_rs(&parts[1]));
    }
    first
}

/// Maps a `crate::A::B::...` reference to the module node it targets, or
/// `None` when `A` is not an engine source module (a sibling-crate re-export
/// such as `render`, or a non-module item path).
fn map_target(reference: &str, top_modules: &BTreeSet<String>) -> Option<String> {
    let mut segments = reference.split("::");
    let first = segments.next()?;
    if first == "ecs" || first == "assets" || first == "plugins" {
        return match segments.next() {
            Some(second) => Some(format!("{first}::{second}")),
            None => Some(first.to_string()),
        };
    }
    if top_modules.contains(first) {
        return Some(first.to_string());
    }
    None
}

fn module_name(path: &Path) -> String {
    if path.is_dir() {
        path.file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_default()
    } else {
        strip_rs(&path.file_name().unwrap_or_default().to_string_lossy())
    }
}

fn strip_rs(name: &str) -> String {
    name.strip_suffix(".rs").unwrap_or(name).to_string()
}

fn display_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

/// Finds the segment path following each `crate::`, e.g. `ecs::camera::foo`.
fn crate_refs(text: &str) -> Vec<String> {
    let bytes = text.as_bytes();
    let needle = b"crate::";
    let mut references = Vec::new();
    let mut index = 0;
    while index + needle.len() <= bytes.len() {
        if &bytes[index..index + needle.len()] == needle {
            let mut end = index + needle.len();
            let mut path = String::new();
            loop {
                let segment_start = end;
                while end < bytes.len() && is_ident_byte(bytes[end]) {
                    end += 1;
                }
                if end == segment_start {
                    break;
                }
                if !path.is_empty() {
                    path.push_str("::");
                }
                path.push_str(&text[segment_start..end]);
                if end + 2 <= bytes.len() && &bytes[end..end + 2] == b"::" {
                    end += 2;
                } else {
                    break;
                }
            }
            if !path.is_empty() {
                references.push(path);
            }
            index = end.max(index + needle.len());
        } else {
            index += 1;
        }
    }
    references
}

fn is_ident_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_'
}

fn strip(source: &str) -> String {
    strip_comments(&strip_test_mods(source))
}

/// Removes `#[cfg(test)] mod name { ... }` blocks so test-only references do
/// not count as real dependency edges.
fn strip_test_mods(source: &str) -> String {
    let mut output = String::new();
    let mut rest = source;
    while let Some(marker) = rest.find("#[cfg(test)]") {
        let after = &rest[marker..];
        let Some(brace_offset) = find_mod_brace(after) else {
            break;
        };
        let close = match_brace(&after[brace_offset..]);
        match close {
            Some(close_offset) => {
                output.push_str(&rest[..marker]);
                rest = &after[brace_offset + close_offset..];
            }
            None => break,
        }
    }
    output.push_str(rest);
    output
}

/// Offset of the opening `{` of a `mod name {` that follows a `#[cfg(test)]`,
/// if the attribute does in fact guard a module.
fn find_mod_brace(after_attr: &str) -> Option<usize> {
    let mod_index = after_attr.find("mod ")?;
    let brace_index = after_attr[mod_index..].find('{')?;
    Some(mod_index + brace_index)
}

/// Given a slice starting at `{`, the offset just past the matching `}`.
fn match_brace(from_open: &str) -> Option<usize> {
    let bytes = from_open.as_bytes();
    let mut depth = 0i32;
    for (offset, &byte) in bytes.iter().enumerate() {
        match byte {
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(offset + 1);
                }
            }
            _ => {}
        }
    }
    None
}

fn strip_comments(source: &str) -> String {
    let bytes = source.as_bytes();
    let mut output = String::with_capacity(source.len());
    let mut index = 0;
    while index < bytes.len() {
        if index + 1 < bytes.len() && bytes[index] == b'/' && bytes[index + 1] == b'/' {
            while index < bytes.len() && bytes[index] != b'\n' {
                index += 1;
            }
        } else if index + 1 < bytes.len() && bytes[index] == b'/' && bytes[index + 1] == b'*' {
            index += 2;
            while index + 1 < bytes.len() && !(bytes[index] == b'*' && bytes[index + 1] == b'/') {
                index += 1;
            }
            index = (index + 2).min(bytes.len());
        } else {
            output.push(bytes[index] as char);
            index += 1;
        }
    }
    output
}

/// Tarjan's strongly-connected-components over the module edge set.
fn strongly_connected(edges: &BTreeMap<String, BTreeSet<String>>) -> Vec<Vec<String>> {
    let mut nodes: BTreeSet<String> = BTreeSet::new();
    for (source, targets) in edges {
        nodes.insert(source.clone());
        for target in targets {
            nodes.insert(target.clone());
        }
    }

    let mut state = Tarjan {
        edges,
        index: HashMap::new(),
        low: HashMap::new(),
        on_stack: HashMap::new(),
        stack: Vec::new(),
        counter: 0,
        components: Vec::new(),
    };
    for node in &nodes {
        if !state.index.contains_key(node) {
            state.visit(node);
        }
    }
    state.components
}

struct Tarjan<'a> {
    edges: &'a BTreeMap<String, BTreeSet<String>>,
    index: HashMap<String, usize>,
    low: HashMap<String, usize>,
    on_stack: HashMap<String, bool>,
    stack: Vec<String>,
    counter: usize,
    components: Vec<Vec<String>>,
}

impl Tarjan<'_> {
    fn visit(&mut self, node: &str) {
        self.index.insert(node.to_string(), self.counter);
        self.low.insert(node.to_string(), self.counter);
        self.counter += 1;
        self.stack.push(node.to_string());
        self.on_stack.insert(node.to_string(), true);

        if let Some(targets) = self.edges.get(node) {
            for target in targets {
                if !self.index.contains_key(target) {
                    self.visit(target);
                    let low_target = self.low[target];
                    let entry = self.low.get_mut(node).unwrap();
                    *entry = (*entry).min(low_target);
                } else if *self.on_stack.get(target).unwrap_or(&false) {
                    let index_target = self.index[target];
                    let entry = self.low.get_mut(node).unwrap();
                    *entry = (*entry).min(index_target);
                }
            }
        }

        if self.low[node] == self.index[node] {
            let mut component = Vec::new();
            loop {
                let member = self.stack.pop().unwrap();
                self.on_stack.insert(member.clone(), false);
                component.push(member.clone());
                if member == node {
                    break;
                }
            }
            self.components.push(component);
        }
    }
}
