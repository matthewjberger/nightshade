const PLUGIN_VERBS: [&str; 3] = ["install", "register", "initialize"];

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

const TEMPLATE_MODULES: [&str; 9] = [
    "commands",
    "components",
    "contract",
    "data",
    "plugin",
    "prelude",
    "queries",
    "resources",
    "systems",
];

const WORLD_PARAMETERS: [&str; 3] = ["Commands", "ResMut", "EventWriter"];

const SYSTEM_PARAMETERS: [&str; 5] = ["Query", "Res", "ResMut", "Commands", "EventReader"];

pub struct BandFunction {
    pub file: PathBuf,
    pub name: String,
    pub takes_world: bool,
    pub takes_self: bool,
    pub returns: bool,
    pub reports_a_value: bool,
    pub makes_itself: bool,
    pub reads_something: bool,
}

fn reading_query_aliases() -> BTreeSet<String> {
    let mut found = BTreeSet::new();
    for root in ["crates", "apps", "template"] {
        let mut files = Vec::new();
        crate::collect_rs(Path::new(root), &mut files);
        for file in files {
            let text = crate::strip(&fs::read_to_string(&file).unwrap_or_default());
            for line in text.lines() {
                let line = line.trim();
                let Some(rest) = line.strip_prefix("pub type ") else {
                    continue;
                };
                let Some((name, body)) = rest.split_once('=') else {
                    continue;
                };
                let body = body.trim();
                if !body.starts_with("Query<")
                    || body.contains("&mut ")
                    || body.contains("&'item mut")
                {
                    continue;
                }
                let name = name.split('<').next().unwrap_or_default().trim();
                found.insert(name.to_string());
            }
            found.extend(reading_bundles(&text, &found));
        }
    }
    let mut widened = found.clone();
    for _ in 0..4 {
        let before = widened.len();
        for root in ["crates", "apps", "template"] {
            let mut files = Vec::new();
            crate::collect_rs(Path::new(root), &mut files);
            for file in files {
                let text = crate::strip(&fs::read_to_string(&file).unwrap_or_default());
                widened.extend(reading_bundles(&text, &widened));
            }
        }
        if widened.len() == before {
            break;
        }
    }
    widened
}

fn reading_bundles(text: &str, reading: &BTreeSet<String>) -> BTreeSet<String> {
    let mut found = BTreeSet::new();
    let mut cursor = text;
    while let Some(index) = cursor.find("query_bundle! {") {
        let rest = &cursor[index..];
        let Some(open) = rest.find('{') else { break };
        let mut depth = 0;
        let mut end = open;
        for (offset, character) in rest[open..].char_indices() {
            if character == '{' {
                depth += 1;
            } else if character == '}' {
                depth -= 1;
                if depth == 0 {
                    end = open + offset;
                    break;
                }
            }
        }
        let body = &rest[open..end];
        if let Some(marker) = body.find("pub struct ") {
            let after = &body[marker + "pub struct ".len()..];
            let name: String = after
                .chars()
                .take_while(|character| character.is_alphanumeric() || *character == '_')
                .collect();
            let fields: Vec<String> = body
                .lines()
                .filter_map(|line| line.trim().split_once(':')?.1.trim().split('<').next())
                .map(|value| value.trim().to_string())
                .collect();
            if !fields.is_empty() && fields.iter().all(|field| reading.contains(field)) {
                found.insert(name);
            }
        }
        cursor = &rest[end.max(1)..];
    }
    found
}

pub fn band_functions(band: &str) -> Vec<BandFunction> {
    let reading = reading_query_aliases();
    let mut found = Vec::new();
    for root in ["crates", "apps", "template"] {
        let mut files = Vec::new();
        collect_band_files(Path::new(root), band, &mut files);
        for file in files {
            let text = fs::read_to_string(&file).unwrap_or_default();
            let Ok(parsed) = syn::parse_file(&text) else {
                continue;
            };
            read_items(&parsed.items, &file, &reading, &mut found);
        }
    }
    found
}

fn read_items(
    items: &[syn::Item],
    file: &Path,
    reading: &BTreeSet<String>,
    found: &mut Vec<BandFunction>,
) {
    for item in items {
        match item {
            syn::Item::Fn(function) if is_public(&function.vis) => {
                found.push(read_signature(file, &function.sig, None, reading));
            }
            syn::Item::Impl(block) => {
                for inner in &block.items {
                    if let syn::ImplItem::Fn(function) = inner
                        && is_public(&function.vis)
                    {
                        found.push(read_signature(
                            file,
                            &function.sig,
                            Some(&block.self_ty),
                            reading,
                        ));
                    }
                }
            }
            syn::Item::Mod(module) => {
                if module.attrs.iter().any(names_test) {
                    continue;
                }
                if let Some((_, inner)) = &module.content {
                    read_items(inner, file, reading, found);
                }
            }
            _ => {}
        }
    }
}

fn names_test(attribute: &syn::Attribute) -> bool {
    attribute
        .to_token_stream_string()
        .contains("feature = \"test\"")
        || attribute.path().is_ident("cfg")
            && attribute
                .to_token_stream_string()
                .replace(' ', "")
                .contains("cfg(test)")
}

trait TokenText {
    fn to_token_stream_string(&self) -> String;
}

impl TokenText for syn::Attribute {
    fn to_token_stream_string(&self) -> String {
        use quote::ToTokens;
        self.to_token_stream().to_string()
    }
}

fn is_public(visibility: &syn::Visibility) -> bool {
    matches!(visibility, syn::Visibility::Public(_))
}

fn read_signature(
    file: &Path,
    signature: &syn::Signature,
    owner: Option<&syn::Type>,
    reading: &BTreeSet<String>,
) -> BandFunction {
    use proc_macro2::Span;
    let _ = Span::call_site();
    let mut takes_world = false;
    let mut takes_self = false;
    let mut reads_something = false;
    for input in &signature.inputs {
        match input {
            syn::FnArg::Receiver(_) => takes_self = true,
            syn::FnArg::Typed(typed) => {
                if names_world(&typed.ty, reading) {
                    takes_world = true;
                }
                if is_shared_reference(&typed.ty) {
                    reads_something = true;
                }
            }
        }
    }
    if owner.is_some() && takes_self {
        takes_self = true;
    }
    BandFunction {
        file: file.to_path_buf(),
        name: signature.ident.to_string(),
        takes_world,
        takes_self,
        returns: !matches!(signature.output, syn::ReturnType::Default),
        reports_a_value: reports_a_plain_value(&signature.output),
        makes_itself: returns_itself(&signature.output),
        reads_something,
    }
}

fn reads_only(kind: &syn::Type, reading: &BTreeSet<String>) -> bool {
    match kind {
        syn::Type::Path(path) => path.path.segments.last().is_some_and(|segment| {
            let head = segment.ident.to_string();
            if reading.contains(&head) {
                return true;
            }
            if head != "Query" {
                return false;
            }
            let syn::PathArguments::AngleBracketed(arguments) = &segment.arguments else {
                return false;
            };
            use quote::ToTokens;
            !arguments.to_token_stream().to_string().contains("mut")
        }),
        syn::Type::Paren(inner) => reads_only(&inner.elem, reading),
        syn::Type::Group(inner) => reads_only(&inner.elem, reading),
        _ => false,
    }
}

fn names_world(kind: &syn::Type, reading: &BTreeSet<String>) -> bool {
    match kind {
        syn::Type::Reference(reference) => {
            reference.mutability.is_some() && !reads_only(&reference.elem, reading)
        }
        syn::Type::Path(path) => path
            .path
            .segments
            .iter()
            .any(|segment| WORLD_PARAMETERS.iter().any(|name| segment.ident == *name)),
        syn::Type::Paren(inner) => names_world(&inner.elem, reading),
        syn::Type::Group(inner) => names_world(&inner.elem, reading),
        _ => false,
    }
}

fn is_shared_reference(kind: &syn::Type) -> bool {
    match kind {
        syn::Type::Reference(reference) => reference.mutability.is_none(),
        syn::Type::Paren(inner) => is_shared_reference(&inner.elem),
        syn::Type::Group(inner) => is_shared_reference(&inner.elem),
        _ => false,
    }
}

fn returns_itself(output: &syn::ReturnType) -> bool {
    let syn::ReturnType::Type(_, kind) = output else {
        return false;
    };
    let syn::Type::Path(path) = kind.as_ref() else {
        return false;
    };
    path.path.is_ident("Self")
}

fn reports_a_plain_value(output: &syn::ReturnType) -> bool {
    let syn::ReturnType::Type(_, kind) = output else {
        return false;
    };
    let syn::Type::Path(path) = kind.as_ref() else {
        return true;
    };
    !path
        .path
        .segments
        .last()
        .is_some_and(|segment| segment.ident == "Result")
}

fn collect_band_files(dir: &Path, band: &str, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if path.file_name().is_some_and(|name| name == "target") {
                continue;
            }
            collect_band_files(&path, band, out);
            continue;
        }
        if path.extension().is_none_or(|value| value != "rs") {
            continue;
        }
        if !path.to_string_lossy().replace('\\', "/").contains("/src/") {
            continue;
        }
        let in_band_dir = path
            .parent()
            .and_then(|parent| parent.file_name())
            .is_some_and(|name| name == band);
        let is_band_root = path.file_stem().is_some_and(|stem| stem == band)
            && path
                .parent()
                .and_then(|parent| parent.file_name())
                .is_some_and(|name| name == "src");
        if in_band_dir || is_band_root {
            out.push(path);
        }
    }
}

pub fn queries_that_report_nothing() -> BTreeSet<String> {
    band_functions("queries")
        .into_iter()
        .filter(|function| !function.returns)
        .map(|function| format!("{} {}", display(&function.file), function.name))
        .collect()
}

pub fn queries_that_change_the_world() -> BTreeSet<String> {
    band_functions("queries")
        .into_iter()
        .filter(|function| function.takes_world)
        .map(|function| format!("{} {}", display(&function.file), function.name))
        .collect()
}

pub fn commands_that_only_report() -> BTreeSet<String> {
    band_functions("commands")
        .into_iter()
        .filter(|function| {
            !function.takes_world
                && !function.takes_self
                && !function.makes_itself
                && function.reads_something
                && function.reports_a_value
        })
        .map(|function| format!("{} {}", display(&function.file), function.name))
        .collect()
}

pub fn unrecorded(faults: &BTreeSet<String>, recorded: &BTreeSet<String>) -> Vec<String> {
    faults
        .iter()
        .filter(|entry| !recorded.contains(*entry))
        .cloned()
        .collect()
}

pub fn stale(faults: &BTreeSet<String>, recorded: &BTreeSet<String>) -> Vec<String> {
    recorded
        .iter()
        .filter(|entry| !faults.contains(*entry))
        .cloned()
        .collect()
}

pub fn crates_with_a_module_outside_the_template(recorded: &BTreeSet<String>) -> Vec<String> {
    let mut reported = Vec::new();
    for dir in crate::plugin_crate_dirs() {
        let name = crate::crate_name(&dir);
        let text = fs::read_to_string(dir.join("src/lib.rs")).unwrap_or_default();
        for line in text.lines() {
            let line = line.trim();
            let rest = line
                .strip_prefix("pub mod ")
                .or_else(|| line.strip_prefix("mod "));
            let Some(rest) = rest else {
                continue;
            };
            if !rest.ends_with(';') {
                continue;
            }
            let module = rest.trim_end_matches(';').trim();
            if !module
                .chars()
                .all(|character| character.is_ascii_lowercase() || character == '_')
            {
                continue;
            }
            if TEMPLATE_MODULES.contains(&module) {
                continue;
            }
            let entry = format!("{name}: {module}");
            if !recorded.contains(&entry) {
                reported.push(entry);
            }
        }
    }
    reported.sort();
    reported.dedup();
    reported
}

pub fn stale_module_records(recorded: &BTreeSet<String>) -> Vec<String> {
    let mut live = BTreeSet::new();
    for dir in crate::plugin_crate_dirs() {
        let name = crate::crate_name(&dir);
        let text = fs::read_to_string(dir.join("src/lib.rs")).unwrap_or_default();
        for line in text.lines() {
            let line = line.trim();
            let rest = line
                .strip_prefix("pub mod ")
                .or_else(|| line.strip_prefix("mod "));
            if let Some(rest) = rest
                && rest.ends_with(';')
            {
                live.insert(format!("{name}: {}", rest.trim_end_matches(';').trim()));
            }
        }
    }
    recorded
        .iter()
        .filter(|entry| !live.contains(*entry))
        .cloned()
        .collect()
}

pub fn plugin_files_holding_more_than_a_plugin() -> Vec<String> {
    let mut reported = Vec::new();
    for dir in crate::plugin_crate_dirs()
        .into_iter()
        .chain(crate::framework_crate_dirs())
    {
        let path = dir.join("src/plugin.rs");
        let text = fs::read_to_string(&path).unwrap_or_default();
        let Ok(parsed) = syn::parse_file(&text) else {
            continue;
        };
        for item in &parsed.items {
            let fault = match item {
                syn::Item::Fn(function) => {
                    let name = function.sig.ident.to_string();
                    let installs = PLUGIN_VERBS.iter().any(|verb| name.starts_with(verb));
                    (!installs)
                        .then(|| format!("{name} is not an install, a register or an initialize"))
                }
                syn::Item::Enum(declared) => {
                    Some(format!("{} is a value and belongs in data", declared.ident))
                }
                syn::Item::Struct(declared) => {
                    let name = declared.ident.to_string();
                    (!name.ends_with("Plugin"))
                        .then(|| format!("{name} is a value and belongs in its own band"))
                }
                _ => None,
            };
            if let Some(fault) = fault {
                reported.push(format!("{} {fault}", display(&path)));
            }
        }
    }
    reported.sort();
    reported
}

pub fn plugin_files_holding_a_system() -> Vec<String> {
    let mut reported = Vec::new();
    for dir in crate::plugin_crate_dirs()
        .into_iter()
        .chain(crate::framework_crate_dirs())
    {
        let path = dir.join("src/plugin.rs");
        let text = fs::read_to_string(&path).unwrap_or_default();
        let Ok(parsed) = syn::parse_file(&text) else {
            continue;
        };
        for item in &parsed.items {
            let syn::Item::Fn(function) = item else {
                continue;
            };
            let names_parameter = function.sig.inputs.iter().any(|input| {
                let syn::FnArg::Typed(typed) = input else {
                    return false;
                };
                names_system_parameter(&typed.ty)
            });
            if names_parameter {
                reported.push(format!("{} {}", display(&path), function.sig.ident));
            }
        }
    }
    reported.sort();
    reported
}

fn names_system_parameter(kind: &syn::Type) -> bool {
    match kind {
        syn::Type::Path(path) => path
            .path
            .segments
            .iter()
            .any(|segment| SYSTEM_PARAMETERS.iter().any(|name| segment.ident == *name)),
        syn::Type::Reference(reference) => names_system_parameter(&reference.elem),
        syn::Type::Paren(inner) => names_system_parameter(&inner.elem),
        syn::Type::Group(inner) => names_system_parameter(&inner.elem),
        _ => false,
    }
}

pub fn display(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

pub fn recorded(file: &str) -> BTreeSet<String> {
    let Ok(text) = fs::read_to_string(Path::new("tools/dag-check").join(file)) else {
        return BTreeSet::new();
    };
    text.lines()
        .filter(|line| !line.starts_with('#') && !line.trim().is_empty())
        .map(|line| line.trim().to_string())
        .collect()
}

const COMPONENT_CALLS: [&str; 13] = [
    "get",
    "get_mut",
    "has",
    "component_key",
    "add_tag_type",
    "remove_tag_type",
    "has_tag_type",
    "query_tag_type",
    "tag_set_type",
    "query_entities",
    "entity_has_components",
    "entity_has_any",
    "spawn_entities",
];

const AMBIGUOUS_CALLS: [&str; 3] = ["get", "get_mut", "has"];

const COMPONENT_FILTERS: [&str; 4] = ["with", "without", "changed", "added"];

const COMPONENT_QUERIES: [&str; 4] = ["query", "query_ref", "for_each", "for_each_mut"];

const RESOURCE_CALLS: [&str; 4] = [
    "res",
    "res_mut",
    "resource_or_insert_default",
    "get_resource",
];

const COMPONENT_TYPES: [&str; 3] = ["Query", "With", "Without"];

const RESOURCE_TYPES: [&str; 2] = ["Res", "ResMut"];

#[derive(Default)]
struct Named {
    components: BTreeSet<String>,
    resources: BTreeSet<String>,
}

impl<'ast> syn::visit::Visit<'ast> for Named {
    fn visit_expr_method_call(&mut self, call: &'ast syn::ExprMethodCall) {
        if let Some(turbofish) = &call.turbofish {
            let method = call.method.to_string();
            let names = names_in(&turbofish.args);
            let on_the_map = reads_the_resource_map(&call.receiver);
            if RESOURCE_CALLS.contains(&method.as_str())
                || (on_the_map && AMBIGUOUS_CALLS.contains(&method.as_str()))
            {
                self.resources.extend(names);
            } else if COMPONENT_CALLS.contains(&method.as_str())
                || COMPONENT_FILTERS.contains(&method.as_str())
                || COMPONENT_QUERIES.contains(&method.as_str())
            {
                self.components.extend(names);
            }
        }
        syn::visit::visit_expr_method_call(self, call);
    }

    fn visit_type_path(&mut self, path: &'ast syn::TypePath) {
        if let Some(segment) = path.path.segments.last()
            && let syn::PathArguments::AngleBracketed(arguments) = &segment.arguments
        {
            let head = segment.ident.to_string();
            let names = names_in(&arguments.args);
            if COMPONENT_TYPES.contains(&head.as_str()) {
                self.components.extend(names);
            } else if RESOURCE_TYPES.contains(&head.as_str()) {
                self.resources.extend(names);
            }
        }
        syn::visit::visit_type_path(self, path);
    }
}

fn reads_the_resource_map(receiver: &syn::Expr) -> bool {
    use quote::ToTokens;
    let text = receiver.to_token_stream().to_string().replace(' ', "");
    text.contains("resources") || text.contains("resource_map") || text == "map"
}

fn names_in(
    arguments: &syn::punctuated::Punctuated<syn::GenericArgument, syn::token::Comma>,
) -> BTreeSet<String> {
    let mut found = BTreeSet::new();
    for argument in arguments {
        if let syn::GenericArgument::Type(kind) = argument {
            gather_names(kind, &mut found);
        }
    }
    found
}

fn gather_names(kind: &syn::Type, found: &mut BTreeSet<String>) {
    match kind {
        syn::Type::Reference(reference) => gather_names(&reference.elem, found),
        syn::Type::Paren(inner) => gather_names(&inner.elem, found),
        syn::Type::Group(inner) => gather_names(&inner.elem, found),
        syn::Type::Tuple(tuple) => {
            for element in &tuple.elems {
                gather_names(element, found);
            }
        }
        syn::Type::Path(path) => {
            let Some(segment) = path.path.segments.last() else {
                return;
            };
            let head = segment.ident.to_string();
            if head == "Option" || head == "Vec" {
                if let syn::PathArguments::AngleBracketed(arguments) = &segment.arguments {
                    found.extend(names_in(&arguments.args));
                }
                return;
            }
            found.insert(head);
        }
        _ => {}
    }
}

fn declared_types(dir: &Path) -> std::collections::BTreeMap<String, PathBuf> {
    let mut found = std::collections::BTreeMap::new();
    let mut files = Vec::new();
    crate::collect_rs(&dir.join("src"), &mut files);
    for file in files {
        let text = fs::read_to_string(&file).unwrap_or_default();
        let Ok(parsed) = syn::parse_file(&text) else {
            continue;
        };
        collect_declarations(&parsed.items, &file, &mut found);
    }
    found
}

fn collect_declarations(
    items: &[syn::Item],
    file: &Path,
    found: &mut std::collections::BTreeMap<String, PathBuf>,
) {
    for item in items {
        match item {
            syn::Item::Struct(entry) if is_public(&entry.vis) => {
                found.insert(entry.ident.to_string(), file.to_path_buf());
            }
            syn::Item::Enum(entry) if is_public(&entry.vis) => {
                found.insert(entry.ident.to_string(), file.to_path_buf());
            }
            syn::Item::Mod(module) => {
                if let Some((_, inner)) = &module.content {
                    collect_declarations(inner, file, found);
                }
            }
            _ => {}
        }
    }
}

fn band_of(file: &Path, dir: &Path) -> String {
    let relative = file
        .strip_prefix(dir.join("src"))
        .unwrap_or(file)
        .to_string_lossy()
        .replace('\\', "/");
    relative
        .split('/')
        .next()
        .unwrap_or_default()
        .trim_end_matches(".rs")
        .to_string()
}

fn types_named_in(dir: &Path) -> Named {
    use syn::visit::Visit;
    let mut named = Named::default();
    let mut files = Vec::new();
    crate::collect_rs(&dir.join("src"), &mut files);
    for file in files {
        let text = crate::strip(&fs::read_to_string(&file).unwrap_or_default());
        let Ok(parsed) = syn::parse_file(&text) else {
            continue;
        };
        named.visit_file(&parsed);
    }
    named
}

fn misplaced(band_wanted: &[&str], pick: fn(&Named) -> &BTreeSet<String>) -> Vec<String> {
    let mut reported = Vec::new();
    for dir in crate::plugin_crate_dirs()
        .into_iter()
        .chain(crate::framework_crate_dirs())
    {
        let declared = declared_types(&dir);
        let named = types_named_in(&dir);
        for name in pick(&named) {
            let Some(file) = declared.get(name) else {
                continue;
            };
            let band = band_of(file, &dir);
            if band_wanted.contains(&band.as_str()) {
                continue;
            }
            reported.push(format!(
                "{} {} (declared in {})",
                crate::crate_name(&dir),
                name,
                band
            ));
        }
    }
    reported.sort();
    reported.dedup();
    reported
}

pub fn components_outside_their_band(recorded: &BTreeSet<String>) -> Vec<String> {
    misplaced(&["components", "contract"], |named| &named.components)
        .into_iter()
        .filter(|entry| !recorded.contains(entry))
        .collect()
}

pub fn resources_outside_their_band(recorded: &BTreeSet<String>) -> Vec<String> {
    misplaced(&["resources", "contract"], |named| &named.resources)
        .into_iter()
        .filter(|entry| !recorded.contains(entry))
        .collect()
}
