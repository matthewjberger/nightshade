use naga_oil::compose::ShaderDefValue;
use nightshade_wgpu::backend::ShaderComposer;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use wgpu::naga::valid::{Capabilities, ValidationFlags, Validator};
use wgpu::naga::{AddressSpace, Binding, BuiltIn, Handle, Module, ShaderStage, Type, TypeInner};

const ENGINE: &str = "crates";

const INTER_STAGE_VARIABLES: u32 = 16;

const STORAGE_BUFFERS_PER_STAGE: u32 = 8;
const UNIFORM_BUFFERS_PER_STAGE: u32 = 12;
const VERTEX_ATTRIBUTES: u32 = 16;
const BIND_GROUPS: u32 = 4;
const COMPUTE_INVOCATIONS: u32 = 256;

fn module_path_of(source: &str) -> Option<String> {
    source.lines().find_map(|line| {
        line.trim()
            .strip_prefix("#define_import_path ")
            .map(|name| name.trim().to_string())
    })
}

fn module_imports(source: &str) -> BTreeSet<String> {
    source
        .lines()
        .filter_map(|line| line.trim().strip_prefix("#import "))
        .flat_map(|rest| {
            let target = rest.trim();
            let target = target.split_whitespace().next().unwrap_or(target);
            let target = target.split("::{").next().unwrap_or(target);
            let parent = target
                .rsplit_once("::")
                .map(|(head, _)| head.to_string())
                .filter(|head| !head.is_empty());
            std::iter::once(target.to_string()).chain(parent)
        })
        .collect()
}

fn publish_in_dependency_order(composer: &ShaderComposer, modules: &BTreeSet<PathBuf>) {
    let mut pending: Vec<(String, BTreeSet<String>, String, String)> = Vec::new();
    for path in modules {
        let Some(file_name) = path
            .file_name()
            .map(|name| name.to_string_lossy().to_string())
        else {
            continue;
        };
        let Ok(source) = fs::read_to_string(path) else {
            continue;
        };
        let Some(name) = module_path_of(&source) else {
            continue;
        };
        let imports = module_imports(&source);
        pending.push((name, imports, source, file_name));
    }

    let declared: BTreeSet<String> = pending.iter().map(|entry| entry.0.clone()).collect();
    let mut published: BTreeSet<String> = BTreeSet::new();
    while !pending.is_empty() {
        let ready: Vec<usize> = pending
            .iter()
            .enumerate()
            .filter(|(_, (_, imports, _, _))| {
                imports
                    .iter()
                    .all(|import| published.contains(import) || !declared.contains(import))
            })
            .map(|(index, _)| index)
            .collect();
        if ready.is_empty() {
            for (_, _, source, file_name) in pending.drain(..) {
                composer.publish_module(&source, &file_name);
            }
            return;
        }
        for index in ready.into_iter().rev() {
            let (name, _, source, file_name) = pending.remove(index);
            composer.publish_module(&source, &file_name);
            published.insert(name);
        }
    }
}

struct Composition {
    label: String,
    sources: Vec<PathBuf>,
}

fn main() -> ExitCode {
    let engine = Path::new(ENGINE);
    if !engine.is_dir() {
        eprintln!("shader-check: run from the repository root; {ENGINE} not found");
        return ExitCode::FAILURE;
    }

    let mut rust_files = Vec::new();
    collect(engine, "rs", &mut rust_files);
    let mut compositions = Vec::new();
    for file in &rust_files {
        compositions.extend(call_sites(file));
    }
    let grouped: BTreeSet<PathBuf> = compositions
        .iter()
        .flat_map(|composition| composition.sources.iter().cloned())
        .collect();
    for file in &rust_files {
        for path in included_shaders(file) {
            if is_composable_module(&path) {
                continue;
            }
            if !grouped.contains(&path) {
                compositions.push(Composition {
                    label: path
                        .file_name()
                        .map(|name| name.to_string_lossy().into_owned())
                        .unwrap_or_default(),
                    sources: vec![path],
                });
            }
        }
    }
    compositions.sort_by(|left, right| left.label.cmp(&right.label));
    compositions.dedup_by(|left, right| left.sources == right.sources);

    if compositions.is_empty() {
        eprintln!("shader-check: found no compile_wgsl call sites, which cannot be right");
        return ExitCode::FAILURE;
    }

    let mut shader_files = Vec::new();
    collect(engine, "wgsl", &mut shader_files);
    let covered: BTreeSet<PathBuf> = compositions
        .iter()
        .flat_map(|composition| composition.sources.iter().cloned())
        .collect();
    let modules: BTreeSet<PathBuf> = shader_files
        .iter()
        .filter(|path| is_composable_module(path))
        .cloned()
        .collect();

    let composer = ShaderComposer::new();
    publish_in_dependency_order(&composer, &modules);
    let all_defs: &[(&str, ShaderDefValue)] = &[("BINDLESS", ShaderDefValue::Bool(true))];

    let mut failures = Vec::new();
    let mut over_budget = Vec::new();
    for composition in &compositions {
        let mut source = String::new();
        let mut missing = false;
        for path in &composition.sources {
            match fs::read_to_string(path) {
                Ok(text) => source.push_str(&text),
                Err(_) => {
                    failures.push((
                        composition.label.clone(),
                        format!("missing {}", path.display()),
                    ));
                    missing = true;
                }
            }
        }
        if missing {
            continue;
        }
        for defs in [&[][..], all_defs] {
            match composer.compose(&composition.label, &source, defs) {
                Ok(module) => {
                    over_budget.extend(over_budget_entries(&composition.label, &module));
                    over_budget.extend(over_limit_entries(&composition.label, &module));
                }
                Err(diagnostics) => {
                    let which = if defs.is_empty() {
                        "no defs"
                    } else {
                        "all defs"
                    };
                    failures.push((
                        format!("{} ({which})", composition.label),
                        diagnostics.trim_end().to_string(),
                    ));
                }
            }
        }
    }
    over_budget.sort();
    over_budget.dedup();

    failures.extend(diverged_items());
    let drift = fragment_entry_drift();

    let unreached: Vec<&PathBuf> = shader_files
        .iter()
        .filter(|path| !covered.contains(*path) && !modules.contains(*path))
        .collect();

    println!(
        "shader-check: {} compositions from {} call sites, {} composable modules",
        compositions.len() * 2,
        compositions.len(),
        modules.len()
    );

    if failures.is_empty() && unreached.is_empty() && over_budget.is_empty() && drift.is_empty() {
        println!("every shader composes, and every shader is reached");
        println!("every stage fits inside the {INTER_STAGE_VARIABLES} inter-stage variables");
        println!("every stage stays inside the storage, uniform, and workgroup budgets");
        println!(
            "the mesh shaders agree, apart from {} recorded divergences",
            accepted_divergences().len()
        );
        println!("their fragment entries differ by the recorded number of lines, and no more");
        return ExitCode::SUCCESS;
    }

    if !unreached.is_empty() {
        eprintln!();
        eprintln!("no call site names these shaders, so nothing composes them:");
        for path in &unreached {
            eprintln!("  {}", path.display());
        }
        eprintln!("delete them, or reach them from a pass");
    }

    if !drift.is_empty() {
        eprintln!();
        eprintln!("a fragment entry drifted from the one it is meant to match:");
        for (label, found, recorded) in &drift {
            eprintln!("  {label}: {found} differing lines, {recorded} recorded");
        }
        eprintln!(
            "the static and skinned shaders shade from one material table and their fragment entries are meant to read the same; close the difference, or record the new number in tools/shader-check/mesh-divergence.txt with the reason it stands"
        );
    }

    if !over_budget.is_empty() {
        eprintln!();
        eprintln!("a stage asks a device for more than it grants:");
        for line in &over_budget {
            eprintln!("  {line}");
        }
        eprintln!(
            "none of this fails to compose, so it lands at create_render_pipeline on the frame the pass first draws; pack two of them into one, merge the buffers, or derive the value where it is read"
        );
    }

    for (label, diagnostics) in &failures {
        eprintln!("{label} does not compose:");
        for line in diagnostics.lines() {
            eprintln!("    {line}");
        }
        eprintln!();
    }
    if !failures.is_empty() {
        eprintln!("{} composition failures", failures.len());
    }
    ExitCode::FAILURE
}

fn over_budget_entries(label: &str, module: &Module) -> Vec<String> {
    let mut out = Vec::new();
    for entry in &module.entry_points {
        let mut bindings = Vec::new();
        match entry.stage {
            ShaderStage::Vertex => {
                if let Some(result) = &entry.function.result {
                    push_bindings(module, result.ty, result.binding.as_ref(), &mut bindings);
                }
            }
            ShaderStage::Fragment => {
                for argument in &entry.function.arguments {
                    push_bindings(
                        module,
                        argument.ty,
                        argument.binding.as_ref(),
                        &mut bindings,
                    );
                }
            }
            _ => continue,
        }

        let mut budget = INTER_STAGE_VARIABLES;
        let mut used = 0;
        for binding in &bindings {
            match binding {
                Binding::Location { .. } => used += 1,
                Binding::BuiltIn(builtin) => {
                    if entry.stage == ShaderStage::Fragment {
                        budget = budget.saturating_sub(builtin_share(*builtin));
                    }
                }
            }
        }

        if used > budget {
            let (stage, kind) = if entry.stage == ShaderStage::Vertex {
                ("vertex", "outputs")
            } else {
                ("fragment", "inputs")
            };
            out.push(format!(
                "{label}: {stage} entry `{}` declares {used} user-defined {kind}, and its built-ins leave {budget} of the {INTER_STAGE_VARIABLES}",
                entry.name
            ));
        }
    }
    out
}

fn over_limit_entries(label: &str, module: &Module) -> Vec<String> {
    let mut validator = Validator::new(ValidationFlags::all(), Capabilities::all());
    let Ok(info) = validator.validate(module) else {
        return Vec::new();
    };

    let mut out = Vec::new();
    let mut groups = 0;
    for (_, variable) in module.global_variables.iter() {
        if let Some(binding) = &variable.binding {
            groups = groups.max(binding.group + 1);
            if binding.group >= BIND_GROUPS {
                out.push(format!(
                    "{label}: a binding sits in group {}, and a device grants {BIND_GROUPS} groups",
                    binding.group
                ));
            }
        }
    }
    let _ = groups;

    for (index, entry) in module.entry_points.iter().enumerate() {
        let usage = info.get_entry_point(index);
        let mut storage = 0;
        let mut uniform = 0;
        for (handle, variable) in module.global_variables.iter() {
            if usage[handle].is_empty() {
                continue;
            }
            match variable.space {
                AddressSpace::Storage { .. } => storage += 1,
                AddressSpace::Uniform => uniform += 1,
                _ => {}
            }
        }
        if storage > STORAGE_BUFFERS_PER_STAGE {
            out.push(format!(
                "{label}: `{}` reads {storage} storage buffers, and a stage gets {STORAGE_BUFFERS_PER_STAGE}",
                entry.name
            ));
        }
        if uniform > UNIFORM_BUFFERS_PER_STAGE {
            out.push(format!(
                "{label}: `{}` reads {uniform} uniform buffers, and a stage gets {UNIFORM_BUFFERS_PER_STAGE}",
                entry.name
            ));
        }

        if entry.stage == ShaderStage::Compute {
            let invocations =
                entry.workgroup_size[0] * entry.workgroup_size[1] * entry.workgroup_size[2];
            if invocations > COMPUTE_INVOCATIONS {
                out.push(format!(
                    "{label}: `{}` asks for {invocations} invocations in a workgroup, and a device grants {COMPUTE_INVOCATIONS}",
                    entry.name
                ));
            }
        }

        if entry.stage == ShaderStage::Vertex {
            let mut bindings = Vec::new();
            for argument in &entry.function.arguments {
                push_bindings(
                    module,
                    argument.ty,
                    argument.binding.as_ref(),
                    &mut bindings,
                );
            }
            let attributes = bindings
                .iter()
                .filter(|binding| matches!(binding, Binding::Location { .. }))
                .count() as u32;
            if attributes > VERTEX_ATTRIBUTES {
                out.push(format!(
                    "{label}: `{}` takes {attributes} vertex attributes, and a device grants {VERTEX_ATTRIBUTES}",
                    entry.name
                ));
            }
        }
    }
    out
}

fn push_bindings(
    module: &Module,
    ty: Handle<Type>,
    binding: Option<&Binding>,
    out: &mut Vec<Binding>,
) {
    if let Some(binding) = binding {
        out.push(binding.clone());
        return;
    }
    if let TypeInner::Struct { members, .. } = &module.types[ty].inner {
        out.extend(members.iter().filter_map(|member| member.binding.clone()));
    }
}

fn builtin_share(builtin: BuiltIn) -> u32 {
    match builtin {
        BuiltIn::Barycentric { .. } => 3,
        BuiltIn::FrontFacing
        | BuiltIn::SampleIndex
        | BuiltIn::SampleMask
        | BuiltIn::PrimitiveIndex
        | BuiltIn::SubgroupInvocationId
        | BuiltIn::SubgroupSize
        | BuiltIn::ViewIndex
        | BuiltIn::PointCoord => 1,
        _ => 0,
    }
}

fn collect(root: &Path, extension: &str, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(root) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if path.file_name().is_some_and(|name| name == "target") {
                continue;
            }
            collect(&path, extension, out);
        } else if path.extension().is_some_and(|value| value == extension) {
            out.push(normalize(&path));
        }
    }
}

fn call_sites(file: &Path) -> Vec<Composition> {
    let Ok(raw) = fs::read_to_string(file) else {
        return Vec::new();
    };
    let text: String = raw
        .lines()
        .filter(|line| !line.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join(
            "
",
        );
    let directory = file.parent().unwrap_or(Path::new("."));
    let mut found = Vec::new();
    let mut cursor = 0;
    while let Some(offset) = text[cursor..].find("compile_wgsl") {
        let start = cursor + offset;
        cursor = start + "compile_wgsl".len();
        let Some(open) = text[start..].find('(').map(|at| start + at) else {
            continue;
        };
        let mut depth = 0;
        let mut end = open;
        for (index, character) in text[open..].char_indices() {
            match character {
                '(' => depth += 1,
                ')' => {
                    depth -= 1;
                    if depth == 0 {
                        end = open + index;
                        break;
                    }
                }
                _ => {}
            }
        }
        let call = &text[open..end];
        let sources: Vec<PathBuf> = literals(call, "include_str!(")
            .into_iter()
            .map(|relative| normalize(&directory.join(relative)))
            .collect();
        if sources.is_empty() {
            continue;
        }
        let label = literals(call, "")
            .into_iter()
            .find(|value| !value.contains('/') && !value.ends_with(".rs"))
            .unwrap_or_else(|| sources[0].display().to_string());
        found.push(Composition { label, sources });
        cursor = end;
    }
    found
}

fn is_composable_module(path: &Path) -> bool {
    fs::read_to_string(path)
        .unwrap_or_default()
        .contains("#define_import_path")
}

fn included_shaders(file: &Path) -> Vec<PathBuf> {
    let Ok(raw) = fs::read_to_string(file) else {
        return Vec::new();
    };
    let text: String = raw
        .lines()
        .filter(|line| !line.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join(
            "
",
        );
    let directory = file.parent().unwrap_or(Path::new("."));
    literals(&text, "include_str!(")
        .into_iter()
        .filter(|value| value.ends_with(".wgsl"))
        .map(|relative| normalize(&directory.join(relative)))
        .collect()
}

fn literals(text: &str, prefix: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut rest = text;
    loop {
        let start = if prefix.is_empty() {
            match rest.find('"') {
                Some(at) => at,
                None => break,
            }
        } else {
            match rest.find(prefix) {
                Some(at) => match rest[at..].find('"') {
                    Some(quote) => at + quote,
                    None => break,
                },
                None => break,
            }
        };
        let Some(close) = rest[start + 1..].find('"') else {
            break;
        };
        out.push(rest[start + 1..start + 1 + close].to_string());
        rest = &rest[start + 1 + close + 1..];
    }
    out
}

fn normalize(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for part in path.components() {
        match part {
            std::path::Component::ParentDir => {
                out.pop();
            }
            std::path::Component::CurDir => {}
            other => out.push(other.as_os_str()),
        }
    }
    out
}

const SIBLINGS: &[&str] = &[
    "mesh.wgsl",
    "skinned_mesh.wgsl",
    "mesh_oit.wgsl",
    "skinned_mesh_oit.wgsl",
];

fn diverged_items() -> Vec<(String, String)> {
    let accepted = accepted_divergences();
    let mut bodies: BTreeMap<String, Vec<(String, String)>> = BTreeMap::new();
    for name in SIBLINGS {
        let path = Path::new("crates/plugins/compose/nightshade-wgpu/src/renderer/wgpu/shaders")
            .join(name);
        let Ok(text) = fs::read_to_string(&path) else {
            continue;
        };
        for (item, body) in top_level_items(&text) {
            bodies
                .entry(item)
                .or_default()
                .push(((*name).to_string(), body));
        }
    }
    let mut out = Vec::new();
    for (item, copies) in bodies {
        let distinct: BTreeSet<&String> = copies.iter().map(|(_, body)| body).collect();
        if copies.len() < 2 || distinct.len() < 2 || accepted.contains(&item) {
            continue;
        }
        let files = copies
            .iter()
            .map(|(file, _)| file.as_str())
            .collect::<Vec<_>>()
            .join(", ");
        out.push((
            format!("`{item}` in {files}"),
            concat!(
                "is declared more than once with different bodies. Move it to ",
                "mesh_common.wgsl, make the copies agree, or record why it ",
                "cannot be closed in tools/shader-check/mesh-divergence.txt",
            )
            .to_string(),
        ));
    }
    out
}

const SIBLING_PAIRS: &[(&str, &str)] = &[
    ("mesh.wgsl", "skinned_mesh.wgsl"),
    ("mesh_oit.wgsl", "skinned_mesh_oit.wgsl"),
];

fn fragment_entry_drift() -> Vec<(String, usize, usize)> {
    let recorded = recorded_entry_drift();
    let directory = Path::new("crates/plugins/compose/nightshade-wgpu/src/renderer/wgpu/shaders");
    let mut out = Vec::new();
    for (left_name, right_name) in SIBLING_PAIRS {
        let left_text = fs::read_to_string(directory.join(left_name)).unwrap_or_default();
        let right_text = fs::read_to_string(directory.join(right_name)).unwrap_or_default();
        for entry in fragment_entries(&left_text) {
            let Some(left) = entry_body(&left_text, &entry) else {
                continue;
            };
            let Some(right) = entry_body(&right_text, &entry) else {
                continue;
            };
            let found = differing_lines(&left, &right);
            let label = format!("{entry} {left_name} {right_name}");
            let budget = recorded.get(&label).copied().unwrap_or(0);
            if found != budget {
                out.push((label, found, budget));
            }
        }
    }
    out
}

fn fragment_entries(text: &str) -> Vec<String> {
    let lines: Vec<&str> = text.lines().collect();
    let mut out = Vec::new();
    for (index, line) in lines.iter().enumerate() {
        if !line.starts_with("fn ") || index == 0 || !lines[index - 1].starts_with("@fragment") {
            continue;
        }
        let name: String = line["fn ".len()..]
            .chars()
            .take_while(|character| character.is_alphanumeric() || *character == '_')
            .collect();
        out.push(name);
    }
    out
}

fn entry_body(text: &str, name: &str) -> Option<Vec<String>> {
    let opening = format!("fn {name}(");
    let mut out = Vec::new();
    let mut depth = 0i32;
    let mut started = false;
    for line in text.lines() {
        if line.starts_with(&opening) {
            started = true;
        }
        if !started {
            continue;
        }
        let trimmed = line.trim();
        if !trimmed.is_empty() {
            out.push(trimmed.to_string());
        }
        depth += line.matches('{').count() as i32 - line.matches('}').count() as i32;
        if depth == 0 && out.len() > 1 {
            return Some(out);
        }
    }
    None
}

fn differing_lines(left: &[String], right: &[String]) -> usize {
    let mut previous = vec![0usize; right.len() + 1];
    let mut current = vec![0usize; right.len() + 1];
    for left_line in left {
        for (right_index, right_line) in right.iter().enumerate() {
            current[right_index + 1] = if left_line == right_line {
                previous[right_index] + 1
            } else {
                current[right_index].max(previous[right_index + 1])
            };
        }
        std::mem::swap(&mut previous, &mut current);
    }
    left.len() + right.len() - 2 * previous[right.len()]
}

fn recorded_entry_drift() -> BTreeMap<String, usize> {
    let path = Path::new("tools/shader-check/mesh-divergence.txt");
    let Ok(text) = fs::read_to_string(path) else {
        return BTreeMap::new();
    };
    let mut out = BTreeMap::new();
    for line in text.lines() {
        if line.starts_with('#') || line.starts_with(' ') {
            continue;
        }
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() != 4 {
            continue;
        }
        let Ok(count) = parts[3].parse::<usize>() else {
            continue;
        };
        out.insert(format!("{} {} {}", parts[0], parts[1], parts[2]), count);
    }
    out
}

fn accepted_divergences() -> BTreeSet<String> {
    let path = Path::new("tools/shader-check/mesh-divergence.txt");
    let Ok(text) = fs::read_to_string(path) else {
        return BTreeSet::new();
    };
    text.lines()
        .filter(|line| !line.starts_with('#') && !line.starts_with(' ') && !line.is_empty())
        .filter(|line| line.split_whitespace().count() != 4)
        .map(|line| line.trim().to_string())
        .collect()
}

fn top_level_items(text: &str) -> Vec<(String, String)> {
    const LINE_BREAK: &str = "\n";
    let lines: Vec<&str> = text.lines().collect();
    let mut out = Vec::new();
    let mut index = 0;
    while index < lines.len() {
        let line = lines[index];
        let kind = ["const ", "struct ", "fn "]
            .iter()
            .find(|prefix| line.starts_with(*prefix));
        let Some(kind) = kind else {
            index += 1;
            continue;
        };
        let is_entry = index > 0
            && ["@vertex", "@fragment", "@compute"]
                .iter()
                .any(|stage| lines[index - 1].starts_with(stage));
        let rest = &line[kind.len()..];
        let name: String = rest
            .chars()
            .take_while(|character| character.is_alphanumeric() || *character == '_')
            .collect();
        if *kind == "const " {
            out.push((name, line.to_string()));
            index += 1;
            continue;
        }
        if is_entry {
            index += 1;
            continue;
        }
        let mut end = index;
        while end < lines.len() && lines[end] != "}" && lines[end] != "};" {
            end += 1;
        }
        out.push((
            name,
            lines[index..=end.min(lines.len() - 1)].join(LINE_BREAK),
        ));
        index = end + 1;
    }
    out
}
