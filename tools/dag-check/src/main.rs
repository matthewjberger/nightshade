mod bands;
mod counts;

use quote::ToTokens;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

const SRC: &str = "crates/nightshade/src";
const RENDERER: &str = "crates/plugins/compose/nightshade-wgpu/src/renderer";
const DESCRIPTION: &str = "crates/plugins/describe/nightshade-render/src";

fn main() -> ExitCode {
    let src = Path::new(SRC);
    if !src.is_dir() {
        eprintln!("dag-check: run from the repository root; {SRC} not found");
        return ExitCode::FAILURE;
    }

    let stray = whole_crate_when_contract_would_do();
    let plugin_edges = plugin_edges_off_the_ledger();
    let stale_edges = stale_plugin_edges();
    let inner = unrecorded_inner_cycles();
    let upward_bands = bands_pointing_up();
    let window_calls = window_calls_off_the_main_thread();
    let extents = unclamped_texture_extents();
    let stray_worlds = untracked_world_builds();
    let gpu_features = plugins_declaring_gpu_api();
    let backend_reaches = backend_paths_outside_the_facade();
    let description_runs = behavior_in_the_description();
    let component_objects = components_that_are_objects();
    let stale_sweeps = stale_component_sweeps();
    let raw_reads = raw_input_reads_in_games();
    let stale_raw = stale_raw_input_records();
    let framework_below = framework_named_from_below();
    let group_edges = framework_groups_pointing_up();
    let empty_contracts = contract_features_that_gate_nothing();
    let past_contracts = contract_edges_reaching_past_the_contract();
    let without_examples = framework_crates_without_an_example();
    let without_readmes = crates_without_a_readme();
    let without_plugins = plugin_crates_without_a_plugin();
    let bandless_apps = apps_without_a_systems_band();
    let heavy_support = support_crates_that_name_the_engine();
    let stray_ui_names = ui_crates_that_name_no_interface();
    let grid_drift = cluster_grid_copies_that_drifted();
    let module_records = bands::recorded("module-exceptions.txt");
    let silent_records = bands::recorded("silent-queries.txt");
    let component_records = bands::recorded("component-bands.txt");
    let resource_records = bands::recorded("resource-bands.txt");
    let query_records = bands::recorded("query-mutations.txt");
    let command_records = bands::recorded("command-reports.txt");
    let commented = source_files_with_a_comment();
    let silent_faults = bands::queries_that_report_nothing();
    let silent_queries = bands::unrecorded(&silent_faults, &silent_records);
    let unknown_modules = bands::crates_with_a_module_outside_the_template(&module_records);
    let stale_modules = bands::stale_module_records(&module_records);
    let systems_in_plugin = bands::plugin_files_holding_a_system();
    let crowded_plugins = bands::plugin_files_holding_more_than_a_plugin();
    let levelless = plugin_crates_without_a_level();
    let upward_levels = crate_edges_pointing_up();
    let sibling_plugins = sibling_plugin_edges();
    let stray_backends = backends_named_outside_the_drawing_levels();
    let stale_numbers = counts::numbers_the_document_gets_wrong();
    let query_faults = bands::queries_that_change_the_world();
    let mutating_queries = bands::unrecorded(&query_faults, &query_records);
    let command_faults = bands::commands_that_only_report();
    let reporting_commands = bands::unrecorded(&command_faults, &command_records);
    let stale_queries = bands::stale(&query_faults, &query_records);
    let stale_commands = bands::stale(&command_faults, &command_records);
    let stale_silent = bands::stale(&silent_faults, &silent_records);
    let stray_components = bands::components_outside_their_band(&component_records);
    let stray_resources = bands::resources_outside_their_band(&resource_records);

    if stray.is_empty()
        && plugin_edges.is_empty()
        && stale_edges.is_empty()
        && inner.is_empty()
        && upward_bands.is_empty()
        && window_calls.is_empty()
        && extents.is_empty()
        && stray_worlds.is_empty()
        && gpu_features.is_empty()
        && backend_reaches.is_empty()
        && description_runs.is_empty()
        && component_objects.is_empty()
        && stale_sweeps.is_empty()
        && raw_reads.is_empty()
        && stale_raw.is_empty()
        && framework_below.is_empty()
        && group_edges.is_empty()
        && empty_contracts.is_empty()
        && past_contracts.is_empty()
        && without_examples.is_empty()
        && without_readmes.is_empty()
        && without_plugins.is_empty()
        && bandless_apps.is_empty()
        && heavy_support.is_empty()
        && stray_ui_names.is_empty()
        && grid_drift.is_empty()
        && commented.is_empty()
        && silent_queries.is_empty()
        && unknown_modules.is_empty()
        && stale_modules.is_empty()
        && systems_in_plugin.is_empty()
        && crowded_plugins.is_empty()
        && levelless.is_empty()
        && upward_levels.is_empty()
        && sibling_plugins.is_empty()
        && stray_backends.is_empty()
        && mutating_queries.is_empty()
        && reporting_commands.is_empty()
        && stale_queries.is_empty()
        && stale_commands.is_empty()
        && stale_silent.is_empty()
        && stray_components.is_empty()
        && stray_resources.is_empty()
        && stale_numbers.is_empty()
    {
        println!(
            "every engine and plugin crate is a DAG inside, apart from what plugin-cycles.txt records"
        );
        println!("every plugin crate that could take a contract takes one");
        println!(
            "every plugin-feature edge is one plugin-edges.txt records, and each it records is real"
        );
        println!("every texture extent is a literal or clamped");
        println!("every world in the workspace is built tracking");
        println!("only the backends declare a GPU API");
        println!("every backend path names a module its facade publishes");
        println!("nightshade-render holds no system and no GPU API, and its plugin only installs");
        println!(
            "every app component holds one value, apart from the apps component-sweep.txt records"
        );
        println!(
            "every game asks for input through its actions, apart from the apps raw-input.txt records"
        );
        println!("no engine or plugin crate names a framework crate");
        println!("every framework group names only a group below it, and no crate names a sibling");
        println!("every band names only a band below it");
        println!("every contract feature gates a module, and every contract edge stops at it");
        println!("every framework crate carries an example");
        println!("every plugin and framework crate carries a README");
        println!("every crate under crates/plugins carries a plugin");
        println!("every app carries a systems band");
        println!("no crate under crates/support names the engine");
        println!("every nightshade-ui-x crate builds on nightshade-ui");
        println!("every shader copy of the cluster grid agrees with nightshade-render");
        println!("no source file carries a comment");
        println!(
            "every query reports and changes nothing, apart from what query-mutations.txt records"
        );
        println!("every command changes something, apart from what command-reports.txt records");
        println!(
            "every plugin crate declares only the modules of template/, apart from what module-exceptions.txt records"
        );
        println!("no plugin.rs holds a system");
        println!("every plugin.rs holds only its plugin and what installs or registers it");
        println!("every plugin crate declares a level, and names only a level below its own");
        println!("no plugin crate outside compose takes a sibling's plugin");
        println!("only a draw or compose crate names a backend");
        println!(
            "every type used as a component is declared in components, apart from what component-bands.txt records"
        );
        println!(
            "every type used as a resource is declared in resources, apart from what resource-bands.txt records"
        );
        println!("every window call is on a thread window-handle.txt records");
        println!("every number the project file states is the number the tree holds");
        return ExitCode::SUCCESS;
    }

    let bands_report: [(&str, &Vec<String>, &str); 17] = [
        (
            "a query reports nothing, so it is not a query:",
            &silent_queries,
            "a query only reports, so give it a return type, or move it to commands and record it in silent-queries.txt while you drain the list",
        ),
        (
            "a query takes something it can change:",
            &mutating_queries,
            "a query only reports, so take shared references, or move the function to commands and record it in query-mutations.txt while you drain the list",
        ),
        (
            "a command changes nothing, so it is a query:",
            &reporting_commands,
            "a command changes a value, so take what it changes by &mut, or move the function to queries and record it in command-reports.txt while you drain the list",
        ),
        (
            "a plugin crate declares a module template/ does not carry:",
            &unknown_modules,
            "the bands are commands, components, contract, data, plugin, prelude, queries, resources and systems, so fold the module into one of them or record it in module-exceptions.txt",
        ),
        (
            "a type used as a component is declared outside the components band:",
            &stray_components,
            "per-entity data lives in components, so move the type there, or record it in component-bands.txt while you drain the list",
        ),
        (
            "a type used as a resource is declared outside the resources band:",
            &stray_resources,
            "data shared across entities lives in resources, so move the type there, or record it in resource-bands.txt while you drain the list",
        ),
        (
            "the project file states a number the tree does not hold:",
            &stale_numbers,
            "write the number the tree holds, because a count a person keeps by hand goes stale without a reader",
        ),
        (
            "a plugin.rs holds a system:",
            &systems_in_plugin,
            "plugin.rs registers and nothing else, so move the function into systems/",
        ),
        (
            "a plugin crate declares no level, or one that is not a level:",
            &levelless,
            "add [package.metadata.nightshade] level = \"base\" | \"describe\" | \"domain\" | \"compose\" | \"draw\" to the manifest",
        ),
        (
            "a plugin crate names a crate at a level above its own:",
            &upward_levels,
            "a crate names only a level below its own, so move the type down or move the crate up",
        ),
        (
            "a plugin crate takes a sibling's plugin:",
            &sibling_plugins,
            "take the sibling's contract instead, because only a compose crate may run another crate's plugin",
        ),
        (
            "a crate names a backend from outside the levels that draw:",
            &stray_backends,
            "a backend carries no contract feature, so naming it links the whole renderer, and only a crate under crates/plugins/draw or crates/plugins/compose may do that, while everything else reads the frame description through nightshade-render",
        ),
        (
            "a plugin.rs holds more than its plugin:",
            &crowded_plugins,
            "plugin.rs registers and nothing else, so move the item into data, queries, commands or systems",
        ),
        (
            "tools/dag-check/query-mutations.txt records a query that is gone or clean:",
            &stale_queries,
            "drop the line, so the list only ever shrinks",
        ),
        (
            "tools/dag-check/command-reports.txt records a command that is gone or clean:",
            &stale_commands,
            "drop the line, so the list only ever shrinks",
        ),
        (
            "tools/dag-check/silent-queries.txt records a query that is gone or clean:",
            &stale_silent,
            "drop the line, so the list only ever shrinks",
        ),
        (
            "tools/dag-check/module-exceptions.txt records a module that is gone:",
            &stale_modules,
            "drop the line, so the list only ever shrinks",
        ),
    ];
    for (headline, entries, advice) in bands_report {
        if entries.is_empty() {
            continue;
        }
        eprintln!("{headline}");
        for entry in entries {
            eprintln!("  {entry}");
        }
        eprintln!("{advice}");
    }

    if !commented.is_empty() {
        eprintln!("these source files carry a comment:");
        for line in &commented {
            eprintln!("  {line}");
        }
        eprintln!(
            "the source carries no comment of any kind, rustdoc included, so put what it said in docs/notes/ or in a name that does not need it"
        );
    }

    if !grid_drift.is_empty() {
        eprintln!("a shader copy of the cluster grid disagrees with nightshade-render:");
        for line in &grid_drift {
            eprintln!("  {line}");
        }
        eprintln!(
            "nightshade-render/src/data/config/clusters.rs holds the grid, and neither shader language can name a Rust constant, so change every copy together"
        );
    }

    if !without_examples.is_empty() {
        eprintln!("these framework crates carry no example:");
        for line in &without_examples {
            eprintln!("  {line}");
        }
        eprintln!(
            "the example is what earns a framework crate its place, so write one at examples/<name>/ or move the crate back into the app that wants it"
        );
    }

    if !without_readmes.is_empty() {
        eprintln!("these crates carry no README:");
        for line in &without_readmes {
            eprintln!("  {line}");
        }
        eprintln!(
            "a crate says what it is beside its own source, so write README.md next to the Cargo.toml"
        );
    }

    if !bandless_apps.is_empty() {
        eprintln!("these apps carry no systems band:");
        for line in &bandless_apps {
            eprintln!("  {line}");
        }
        eprintln!(
            "an app keeps the bands of template/, so put the frame behavior in src/systems/<name>.rs rather than in lib.rs"
        );
    }

    if !without_plugins.is_empty() {
        eprintln!("these crates sit under crates/plugins and carry no plugin:");
        for line in &without_plugins {
            eprintln!("  {line}");
        }
        eprintln!(
            "a plugin is what a composable engine piece is, so write plugin.rs, or move the crate to crates/support when it names no engine"
        );
    }

    if !heavy_support.is_empty() {
        eprintln!("these crates sit under crates/support and name the engine:");
        for line in &heavy_support {
            eprintln!("  {line}");
        }
        eprintln!(
            "a support crate stands on its own and a plugin composes it, so take the engine out or move the crate back under crates/plugins with a plugin"
        );
    }

    if !stray_ui_names.is_empty() {
        eprintln!("these crates are named for the retained interface and never name it:");
        for line in &stray_ui_names {
            eprintln!("  {line}");
        }
        eprintln!(
            "the nightshade-ui- prefix says a crate builds on nightshade-ui, so take the prefix off or take the dependency"
        );
    }

    if !empty_contracts.is_empty() {
        eprintln!("these crates declare a contract feature that gates nothing:");
        for line in &empty_contracts {
            eprintln!("  {line}");
        }
    }

    if !past_contracts.is_empty() {
        eprintln!("these contract edges name a module the other crate keeps behind its plugin:");
        for line in &past_contracts {
            eprintln!("  {line}");
        }
    }

    if !group_edges.is_empty() {
        eprintln!("a framework crate names a crate it may not:");
        for entry in &group_edges {
            eprintln!("  {entry}");
        }
        eprintln!(
            "the groups run base, mechanics, interface, tools, and a crate names only a group below its own, so move the shared type into base and announce the rest"
        );
    }

    if !framework_below.is_empty() {
        eprintln!("a crate below the framework level names a crate at it:");
        for entry in &framework_below {
            eprintln!("  {entry}");
        }
        eprintln!(
            "a framework crate takes plugin contracts, and an engine or plugin crate never takes a framework crate, so move what both want down into a plugin contract"
        );
    }

    if !window_calls.is_empty() {
        eprintln!("a file calls the window outside the threads window-handle.txt records:");
        for entry in &window_calls {
            eprintln!("  {entry}");
        }
        eprintln!(
            "a window call belongs on the thread that owns the event loop, so name the MainThread parameter in the system that makes it, and record the file"
        );
    }

    if !upward_bands.is_empty() {
        eprintln!("a band names one above it:");
        for entry in &upward_bands {
            eprintln!("  {entry}");
        }
        eprintln!(
            "the bands run contract, components, resources, data, queries, commands, systems, and each names only what is below it, so move the item down to the band that both want"
        );
    }

    if !component_objects.is_empty() {
        eprintln!("a component gathers several values under the name of a thing:");
        for entry in &component_objects {
            eprintln!("  {entry}");
        }
        eprintln!(
            "a component holds one value, so write one newtype for each field and select on the components an entity carries, as docs/antipatterns.md says"
        );
    }

    if !stale_sweeps.is_empty() {
        eprintln!("tools/dag-check/component-sweep.txt records an app that is already clean:");
        for entry in &stale_sweeps {
            eprintln!("  {entry}");
        }
        eprintln!("drop the line, so the list only ever shrinks");
    }

    if !raw_reads.is_empty() {
        eprintln!("a game reads a key or a pad button by name:");
        for entry in &raw_reads {
            eprintln!("  {entry}");
        }
        eprintln!(
            "a game declares what a person asks for as an Action enum and reads Actions<A> from nightshade-input, so bind the key there and ask for the action"
        );
    }

    if !stale_raw.is_empty() {
        eprintln!("tools/dag-check/raw-input.txt records an app that is already clean:");
        for entry in &stale_raw {
            eprintln!("  {entry}");
        }
        eprintln!("drop the line, so the list only ever shrinks");
    }

    if !description_runs.is_empty() {
        eprintln!("nightshade-render holds something that runs or names a GPU API:");
        for entry in &description_runs {
            eprintln!("  {entry}");
        }
        eprintln!(
            "it is the description both backends read, so a system belongs in nightshade-graphics and a GPU type in the backend that owns it, and plugin.rs may only install what the description owns"
        );
    }

    if !backend_reaches.is_empty() {
        eprintln!("a crate names a backend path the facade does not publish:");
        for entry in &backend_reaches {
            eprintln!("  {entry}");
        }
        eprintln!(
            "the backend module in the crate root lists what the renderer supports, so add the module there or reach the renderer through hooks, pre_render, or render_driver"
        );
    }

    if !gpu_features.is_empty() {
        eprintln!("a crate declares a GPU API that gpu-api-declarations.txt does not record:");
        for name in &gpu_features {
            eprintln!("  {name}");
        }
        eprintln!(
            "a backend owns its API and re-exports it, so reach it through nightshade_wgpu::wgpu or nightshade_vulkan::ash, and record the crate only when the runtime gives it no other way"
        );
    }

    if !stray_worlds.is_empty() {
        eprintln!("a world is built without tracking what changes in it:");
        for entry in &stray_worlds {
            eprintln!("  {entry}");
        }
        eprintln!(
            "transform propagation and the render sync both read the world by diffing it, so build it with nightshade_ecs::World::tracking()"
        );
    }

    if !extents.is_empty() {
        eprintln!("a texture is allocated from a value that was never clamped:");
        for entry in &extents {
            eprintln!("  {entry}");
        }
        eprintln!(
            "wrap the extent in texture_extent::texture_extent so an out-of-range size cannot reach create_texture"
        );
    }

    if !inner.is_empty() {
        eprintln!("a crate has a module dependency cycle inside it:");
        for (plugin, component) in &inner {
            eprintln!("  {plugin}: {component:?}");
        }
        eprintln!(
            "break the cycle by moving the type or the function that points the wrong way, or record it in tools/dag-check/plugin-cycles.txt"
        );
    }

    if !stray.is_empty() {
        eprintln!("a plugin crate takes another whole when its contract would do:");
        for entry in &stray {
            eprintln!("  {entry}");
        }
        eprintln!(
            "the crate names no module that sits behind the other crate's plugin feature, so declare the dependency by path with default-features = false and features = [\"contract\"] and it will stop linking the other capability"
        );
    }

    if !plugin_edges.is_empty() {
        eprintln!("a crate takes another plugin's behavior through its plugin feature:");
        for entry in &plugin_edges {
            eprintln!("  {entry}");
        }
        eprintln!(
            "only a composition root may compose another plugin, so take the contract with default-features = false and features = [\"contract\"], or record the edge in tools/dag-check/plugin-edges.txt if the crate really must compose it"
        );
    }

    if !stale_edges.is_empty() {
        eprintln!("tools/dag-check/plugin-edges.txt records an edge no manifest carries:");
        for entry in &stale_edges {
            eprintln!("  {entry}");
        }
        eprintln!("the crate no longer composes that plugin, so drop the line from the ledger");
    }

    ExitCode::FAILURE
}

pub(crate) fn crate_dirs(root: &str) -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    let Ok(entries) = fs::read_dir(root) else {
        return dirs;
    };
    for entry in entries.flatten() {
        if entry.path().join("src").is_dir() {
            dirs.push(entry.path());
        }
    }
    dirs.sort();
    dirs
}

pub(crate) fn plugin_crate_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    for level in LEVELS {
        dirs.extend(crate_dirs(&format!("crates/plugins/{level}")));
    }
    dirs.sort();
    dirs
}

pub(crate) fn framework_crate_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    let Ok(entries) = fs::read_dir("crates/framework") else {
        return dirs;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.join("src").is_dir() {
            dirs.push(path);
            continue;
        }
        if path.is_dir() {
            dirs.extend(crate_dirs(&path.to_string_lossy()));
        }
    }
    dirs.sort();
    dirs
}

fn framework_group(dir: &Path) -> Option<String> {
    let parent = dir.parent()?;
    if parent.file_name()? == "framework" {
        return None;
    }
    Some(parent.file_name()?.to_string_lossy().to_string())
}

pub(crate) fn crate_name(dir: &Path) -> String {
    dir.file_name()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_default()
}

fn gated_names(dir: &Path) -> BTreeSet<String> {
    let text = fs::read_to_string(dir.join("src/lib.rs")).unwrap_or_default();
    let mut gated = BTreeSet::new();
    let mut behind_plugin = false;
    for line in text.lines() {
        let line = line.trim();
        if line.starts_with("#[") {
            behind_plugin |= line.contains("feature = \"plugin\"");
            continue;
        }
        if line.is_empty() || line.starts_with("//") {
            continue;
        }
        if !behind_plugin {
            continue;
        }
        if let Some(rest) = line.strip_prefix("pub mod ") {
            gated.insert(rest.trim_end_matches(';').trim().to_string());
        }
        if let Some(rest) = line.strip_prefix("pub use ") {
            for name in rest
                .trim_end_matches(';')
                .rsplit("::")
                .next()
                .unwrap_or_default()
                .trim_matches(|character| "{} ".contains(character))
                .split(',')
            {
                let name = name.trim();
                if !name.is_empty() {
                    gated.insert(name.to_string());
                }
            }
        }
        behind_plugin = false;
    }
    gated
}

fn splits_at_contract(dir: &Path) -> bool {
    fs::read_to_string(dir.join("Cargo.toml"))
        .unwrap_or_default()
        .lines()
        .any(|line| line.trim_start().starts_with("contract = "))
}

fn manifest_dependencies(manifest: &str) -> BTreeMap<String, DependencyEntry> {
    let mut found = BTreeMap::new();
    let Ok(document) = manifest.parse::<toml::Table>() else {
        return found;
    };
    let mut sections: Vec<&toml::Table> = Vec::new();
    if let Some(toml::Value::Table(table)) = document.get("dependencies") {
        sections.push(table);
    }
    if let Some(toml::Value::Table(targets)) = document.get("target") {
        for value in targets.values() {
            if let toml::Value::Table(table) = value
                && let Some(toml::Value::Table(dependencies)) = table.get("dependencies")
            {
                sections.push(dependencies);
            }
        }
    }
    for section in sections {
        for (key, value) in section {
            let mut entry = DependencyEntry {
                name: key.clone(),
                default_features: true,
                features: BTreeSet::new(),
            };
            if let toml::Value::Table(table) = value {
                if let Some(toml::Value::String(package)) = table.get("package") {
                    entry.name = package.clone();
                }
                if let Some(toml::Value::Boolean(default)) = table.get("default-features") {
                    entry.default_features = *default;
                }
                if let Some(toml::Value::Array(features)) = table.get("features") {
                    for feature in features {
                        if let toml::Value::String(feature) = feature {
                            entry.features.insert(feature.clone());
                        }
                    }
                }
            }
            found.insert(entry.name.clone(), entry);
        }
    }
    found
}

struct DependencyEntry {
    name: String,
    default_features: bool,
    features: BTreeSet<String>,
}

fn whole_crate_dependencies(manifest: &str) -> BTreeSet<String> {
    manifest_dependencies(manifest)
        .into_values()
        .filter(|entry| entry.default_features && entry.name.starts_with("nightshade-"))
        .map(|entry| entry.name)
        .collect()
}

fn declared_dependencies(manifest: &str) -> BTreeSet<String> {
    manifest_dependencies(manifest)
        .into_keys()
        .filter(|name| name.starts_with("nightshade-"))
        .collect()
}

struct WindowHandleVisitor {
    locals: BTreeSet<String>,
    calls: bool,
}

const OPTION_METHODS: &[&str] = &[
    "is_some",
    "is_none",
    "as_ref",
    "as_mut",
    "as_deref",
    "clone",
    "take",
    "replace",
    "unwrap_or",
    "unwrap_or_else",
    "unwrap_or_default",
    "iter",
];

impl WindowHandleVisitor {
    fn names_the_window(expression: &syn::Expr) -> bool {
        match expression {
            syn::Expr::Path(path) => path
                .path
                .segments
                .last()
                .is_some_and(|segment| segment.ident.to_string().to_lowercase().contains("window")),
            syn::Expr::Field(field) => match &field.member {
                syn::Member::Named(name) => name.to_string().to_lowercase().contains("window"),
                syn::Member::Unnamed(_) => false,
            },
            syn::Expr::MethodCall(call) => call.turbofish.as_ref().is_some_and(|turbofish| {
                turbofish.args.iter().any(|argument| {
                    matches!(argument, syn::GenericArgument::Type(syn::Type::Path(path))
                        if path.path.segments.last().is_some_and(|segment| segment.ident == "Window"))
                })
            }),
            syn::Expr::Reference(inner) => Self::names_the_window(&inner.expr),
            syn::Expr::Unary(inner) => Self::names_the_window(&inner.expr),
            syn::Expr::Paren(inner) => Self::names_the_window(&inner.expr),
            _ => false,
        }
    }

    fn reaches_the_handle(expression: &syn::Expr) -> bool {
        match expression {
            syn::Expr::MethodCall(call)
                if OPTION_METHODS.contains(&call.method.to_string().as_str()) =>
            {
                Self::reaches_the_handle(&call.receiver)
            }
            syn::Expr::Reference(inner) => Self::reaches_the_handle(&inner.expr),
            syn::Expr::Paren(inner) => Self::reaches_the_handle(&inner.expr),
            syn::Expr::Try(inner) => Self::reaches_the_handle(&inner.expr),
            other => Self::names_the_handle(other),
        }
    }

    fn names_the_handle(expression: &syn::Expr) -> bool {
        let syn::Expr::Field(field) = expression else {
            return false;
        };
        let syn::Member::Named(name) = &field.member else {
            return false;
        };
        name == "handle" && Self::names_the_window(&field.base)
    }
}

impl<'ast> syn::visit::Visit<'ast> for WindowHandleVisitor {
    fn visit_local(&mut self, local: &'ast syn::Local) {
        if let Some(init) = &local.init
            && Self::reaches_the_handle(&init.expr)
        {
            for name in pattern_names(&local.pat) {
                self.locals.insert(name);
            }
        }
        syn::visit::visit_local(self, local);
    }

    fn visit_expr_let(&mut self, expression: &'ast syn::ExprLet) {
        if Self::reaches_the_handle(&expression.expr) {
            for name in pattern_names(&expression.pat) {
                self.locals.insert(name);
            }
        }
        syn::visit::visit_expr_let(self, expression);
    }

    fn visit_expr_method_call(&mut self, call: &'ast syn::ExprMethodCall) {
        if Self::reaches_the_handle(&call.receiver) {
            if OPTION_METHODS.contains(&call.method.to_string().as_str()) {
                for argument in &call.args {
                    if let syn::Expr::Closure(closure) = argument {
                        for pattern in &closure.inputs {
                            for name in pattern_names(pattern) {
                                self.locals.insert(name);
                            }
                        }
                    }
                }
            } else {
                self.calls = true;
            }
        }
        if let syn::Expr::Path(path) = &*call.receiver
            && let Some(name) = path.path.get_ident()
            && self.locals.contains(&name.to_string())
        {
            self.calls = true;
        }
        syn::visit::visit_expr_method_call(self, call);
    }
}

fn pattern_names(pattern: &syn::Pat) -> Vec<String> {
    match pattern {
        syn::Pat::Ident(ident) => vec![ident.ident.to_string()],
        syn::Pat::TupleStruct(tuple) => tuple.elems.iter().flat_map(pattern_names).collect(),
        syn::Pat::Reference(inner) => pattern_names(&inner.pat),
        _ => Vec::new(),
    }
}

fn window_calls_off_the_main_thread() -> Vec<String> {
    let recorded: BTreeSet<String> = fs::read_to_string("tools/dag-check/window-handle.txt")
        .unwrap_or_default()
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(|line| line.to_owned())
        .collect();
    let mut files = Vec::new();
    for dir in workspace_source_dirs() {
        collect_rust_files(&dir, &mut files);
    }
    files.sort();
    files.dedup();
    let mut reported = Vec::new();
    let mut seen = BTreeSet::new();
    for file in files {
        let Ok(text) = fs::read_to_string(&file) else {
            continue;
        };
        if !text.contains(".handle") {
            continue;
        }
        let Ok(parsed) = syn::parse_file(&text) else {
            continue;
        };
        let mut visitor = WindowHandleVisitor {
            locals: BTreeSet::new(),
            calls: false,
        };
        syn::visit::Visit::visit_file(&mut visitor, &parsed);
        if !visitor.calls {
            continue;
        }
        let shown = display_path(&file);
        seen.insert(shown.clone());
        if !recorded.contains(&shown) {
            reported.push(shown);
        }
    }
    for line in &recorded {
        if !seen.contains(line) {
            reported.push(format!(
                "{line} no longer calls the window, so drop the line"
            ));
        }
    }
    reported.sort();
    reported.dedup();
    reported
}

fn framework_named_from_below() -> Vec<String> {
    let framework: BTreeSet<String> = framework_crate_dirs()
        .iter()
        .map(|dir| crate_name(dir))
        .collect();
    if framework.is_empty() {
        return Vec::new();
    }
    let mut reported = Vec::new();
    for dir in crate_dirs("crates").into_iter().chain(plugin_crate_dirs()) {
        let name = crate_name(&dir);
        let manifest = fs::read_to_string(dir.join("Cargo.toml")).unwrap_or_default();
        for other in declared_dependencies(&manifest) {
            if framework.contains(&other) {
                reported.push(format!("{name} -> {other} (Cargo.toml)"));
            }
        }
        let mut files = Vec::new();
        collect_rs(&dir.join("src"), &mut files);
        for file in &files {
            let text = strip(&fs::read_to_string(file).unwrap_or_default());
            for other in &framework {
                let stem = format!("{}::", other.replace('-', "_"));
                if text.contains(&stem) {
                    reported.push(format!("{name} -> {other} ({})", display_path(file)));
                    break;
                }
            }
        }
    }
    reported.sort();
    reported.dedup();
    reported
}

fn framework_groups_pointing_up() -> Vec<String> {
    const ORDER: [&str; 4] = ["base", "mechanics", "interface", "tools"];
    let dirs = framework_crate_dirs();
    let mut rank = BTreeMap::new();
    for dir in &dirs {
        let Some(group) = framework_group(dir) else {
            continue;
        };
        let Some(index) = ORDER.iter().position(|known| *known == group) else {
            continue;
        };
        rank.insert(crate_name(dir), index);
    }
    let mut reported = Vec::new();
    for dir in &dirs {
        let name = crate_name(dir);
        let Some(here) = rank.get(&name).copied() else {
            continue;
        };
        let manifest = fs::read_to_string(dir.join("Cargo.toml")).unwrap_or_default();
        for other in declared_dependencies(&manifest) {
            let Some(there) = rank.get(&other).copied() else {
                continue;
            };
            let sideways = there == here && ORDER[here] == "base";
            if there >= here && !sideways {
                reported.push(format!("{name} -> {other} (Cargo.toml)"));
            }
        }
    }
    reported.sort();
    reported.dedup();
    reported
}

fn whole_crate_when_contract_would_do() -> Vec<String> {
    let dirs: Vec<PathBuf> = plugin_crate_dirs()
        .into_iter()
        .chain(framework_crate_dirs())
        .collect();
    let mut reported = Vec::new();
    for dir in &dirs {
        let manifest = fs::read_to_string(dir.join("Cargo.toml")).unwrap_or_default();
        let taken_whole = whole_crate_dependencies(&manifest);
        let mut files = Vec::new();
        collect_rs(&dir.join("src"), &mut files);
        for other in &dirs {
            let other_name = crate_name(other);
            if other == dir || !taken_whole.contains(&other_name) || !splits_at_contract(other) {
                continue;
            }
            let gated = gated_names(other);
            let stem = format!("{}::", other_name.replace('-', "_"));
            let mut named_any = false;
            let mut named_gated = false;
            for file in &files {
                let text = strip(&fs::read_to_string(file).unwrap_or_default());
                let mut cursor = text.as_str();
                while let Some(index) = cursor.find(&stem) {
                    let rest = &cursor[index + stem.len()..];
                    let module: String = rest
                        .chars()
                        .take_while(|character| character.is_alphanumeric() || *character == '_')
                        .collect();
                    named_any = true;
                    if gated.contains(&module) {
                        named_gated = true;
                    }
                    cursor = &cursor[index + stem.len()..];
                }
            }
            if named_any && !named_gated {
                reported.push(format!("{} -> {}", crate_name(dir), other_name));
            }
        }
    }
    reported.sort();
    reported
}

fn contract_dependencies(manifest: &str) -> BTreeSet<String> {
    manifest_dependencies(manifest)
        .into_values()
        .filter(|entry| {
            !entry.default_features
                && entry.features.contains("contract")
                && !entry.features.contains("plugin")
                && entry.name.starts_with("nightshade-")
        })
        .map(|entry| entry.name)
        .collect()
}

fn contract_features_that_gate_nothing() -> Vec<String> {
    let mut reported = Vec::new();
    for dir in plugin_crate_dirs()
        .into_iter()
        .chain(framework_crate_dirs())
    {
        if !splits_at_contract(&dir) {
            continue;
        }
        let text = fs::read_to_string(dir.join("src/lib.rs")).unwrap_or_default();
        if !text.contains("#[cfg(feature = \"") {
            reported.push(format!("{} (Cargo.toml)", crate_name(&dir)));
        }
    }
    reported.sort();
    reported
}

fn contract_edges_reaching_past_the_contract() -> Vec<String> {
    let dirs: Vec<PathBuf> = plugin_crate_dirs()
        .into_iter()
        .chain(framework_crate_dirs())
        .collect();
    let mut reported = Vec::new();
    for dir in &dirs {
        let manifest = fs::read_to_string(dir.join("Cargo.toml")).unwrap_or_default();
        let by_contract = contract_dependencies(&manifest);
        let mut files = Vec::new();
        collect_rs(&dir.join("src"), &mut files);
        for other in &dirs {
            let other_name = crate_name(other);
            if other == dir || !by_contract.contains(&other_name) {
                continue;
            }
            let gated = gated_names(other);
            if gated.is_empty() {
                continue;
            }
            let stem = format!("{}::", other_name.replace('-', "_"));
            for file in &files {
                let text = strip(&fs::read_to_string(file).unwrap_or_default());
                let mut cursor = text.as_str();
                while let Some(index) = cursor.find(&stem) {
                    let rest = &cursor[index + stem.len()..];
                    let module: String = rest
                        .chars()
                        .take_while(|character| character.is_alphanumeric() || *character == '_')
                        .collect();
                    if gated.contains(&module) {
                        reported.push(format!(
                            "{} -> {}::{} ({})",
                            crate_name(dir),
                            other_name,
                            module,
                            file.display()
                        ));
                    }
                    cursor = &cursor[index + stem.len()..];
                }
            }
        }
    }
    reported.sort();
    reported.dedup();
    reported
}

fn framework_crates_without_an_example() -> Vec<String> {
    let mut reported = Vec::new();
    for dir in framework_crate_dirs() {
        if !dir.join("examples").is_dir() {
            reported.push(crate_name(&dir));
        }
    }
    reported.sort();
    reported
}

fn apps_without_a_systems_band() -> Vec<String> {
    let mut reported = Vec::new();
    for dir in app_crate_dirs() {
        let source = dir.join("src");
        if source.join("systems.rs").is_file() || source.join("systems").is_dir() {
            continue;
        }
        reported.push(crate_name(&dir));
    }
    reported.sort();
    reported
}

fn app_crate_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    for group in [
        "apps",
        "apps/features",
        "apps/games",
        "apps/games/multiplayer",
        "apps/rendering",
        "apps/ui",
        "apps/vulkan",
        "apps/framework",
    ] {
        dirs.extend(crate_dirs(group));
    }
    dirs.sort();
    dirs.dedup();
    dirs
}

fn plugin_crates_without_a_plugin() -> Vec<String> {
    let mut reported = Vec::new();
    for dir in plugin_crate_dirs() {
        if !dir.join("src/plugin.rs").is_file() {
            reported.push(crate_name(&dir));
        }
    }
    reported.sort();
    reported
}

fn support_crates_that_name_the_engine() -> Vec<String> {
    let mut reported = Vec::new();
    for dir in crate_dirs("crates/support") {
        let manifest = fs::read_to_string(dir.join("Cargo.toml")).unwrap_or_default();
        if manifest
            .lines()
            .any(|line| line.trim_start().starts_with("nightshade ="))
        {
            reported.push(crate_name(&dir));
        }
    }
    reported.sort();
    reported
}

fn crates_without_a_readme() -> Vec<String> {
    let mut reported = Vec::new();
    for dir in plugin_crate_dirs()
        .into_iter()
        .chain(framework_crate_dirs())
    {
        let readme = dir.join("README.md");
        let written = fs::read_to_string(&readme).unwrap_or_default();
        if written.trim().is_empty() {
            reported.push(crate_name(&dir));
        }
    }
    reported.sort();
    reported
}

fn ui_crates_that_name_no_interface() -> Vec<String> {
    let mut reported = Vec::new();
    for dir in plugin_crate_dirs()
        .into_iter()
        .chain(framework_crate_dirs())
    {
        let name = crate_name(&dir);
        if !name.starts_with("nightshade-ui-") {
            continue;
        }
        let manifest = fs::read_to_string(dir.join("Cargo.toml")).unwrap_or_default();
        if !manifest_dependencies(&manifest).contains_key("nightshade-ui") {
            reported.push(name);
        }
    }
    reported.sort();
    reported
}

fn cluster_grid_copies_that_drifted() -> Vec<String> {
    let rust =
        fs::read_to_string("crates/plugins/describe/nightshade-render/src/data/config/clusters.rs")
            .unwrap_or_default();
    let read = |text: &str, marker: &str| -> Option<u32> {
        let index = text.find(marker)?;
        let rest = &text[index + marker.len()..];
        let digits: String = rest
            .chars()
            .skip_while(|character| !character.is_ascii_digit())
            .take_while(char::is_ascii_digit)
            .collect();
        digits.parse().ok()
    };
    let axes = [
        ("X", "CLUSTER_GRID_X: u32 ="),
        ("Y", "CLUSTER_GRID_Y: u32 ="),
        ("Z", "CLUSTER_GRID_Z: u32 ="),
    ];
    let copies = [
        (
            "crates/plugins/draw/nightshade-wgpu-shading/src/shaders/mesh_common.wgsl",
            [
                "const CLUSTER_GRID_X: u32 =",
                "const CLUSTER_GRID_Y: u32 =",
                "const CLUSTER_GRID_Z: u32 =",
            ],
        ),
        (
            "crates/plugins/compose/nightshade-vulkan/src/renderer/shaders/scene_common.glsl",
            [
                "#define CLUSTER_X",
                "#define CLUSTER_Y",
                "#define CLUSTER_Z",
            ],
        ),
    ];
    let mut reported = Vec::new();
    for (path, markers) in copies {
        let text = fs::read_to_string(path).unwrap_or_default();
        for (index, (axis, rust_marker)) in axes.iter().enumerate() {
            let Some(wanted) = read(&rust, rust_marker) else {
                continue;
            };
            match read(&text, markers[index]) {
                Some(found) if found == wanted => {}
                Some(found) => reported.push(format!(
                    "{path}: cluster {axis} is {found}, nightshade-render says {wanted}"
                )),
                None => reported.push(format!("{path}: cluster {axis} is missing")),
            }
        }
    }
    reported
}

fn source_files_with_a_comment() -> Vec<String> {
    let shader_extensions = [
        "wgsl", "glsl", "comp", "vert", "frag", "rgen", "rchit", "rmiss", "task", "mesh", "rahit",
        "rint",
    ];
    let mut reported = Vec::new();
    for root in ["crates", "apps", "template", "tools"] {
        let mut files = Vec::new();
        collect_sources(Path::new(root), &mut files, &shader_extensions);
        for file in files {
            let text = fs::read_to_string(&file).unwrap_or_default();
            let rust = file.extension().is_some_and(|value| value == "rs");
            for (number, line) in text.lines().enumerate() {
                let trimmed = line.trim_start();
                let found = if rust {
                    trimmed.starts_with("///") || trimmed.starts_with("//!")
                } else {
                    trimmed.starts_with("//")
                };
                if found {
                    reported.push(format!("{}:{}", file.display(), number + 1));
                    break;
                }
            }
        }
    }
    reported.sort();
    reported
}

fn collect_sources(dir: &Path, out: &mut Vec<PathBuf>, shader_extensions: &[&str]) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if path.file_name().is_some_and(|name| name == "target") {
                continue;
            }
            collect_sources(&path, out, shader_extensions);
            continue;
        }
        let Some(extension) = path.extension().and_then(|value| value.to_str()) else {
            continue;
        };
        if extension == "rs" || shader_extensions.contains(&extension) {
            out.push(path);
        }
    }
}

fn recorded_plugin_edges() -> BTreeSet<String> {
    let path = Path::new("tools/dag-check/plugin-edges.txt");
    let Ok(text) = fs::read_to_string(path) else {
        return BTreeSet::new();
    };
    text.lines()
        .filter(|line| !line.starts_with('#') && !line.starts_with(' ') && !line.trim().is_empty())
        .map(|line| line.trim().to_string())
        .collect()
}

fn splitting_crates() -> BTreeMap<String, bool> {
    let mut found = BTreeMap::new();
    for dir in plugin_crate_dirs()
        .into_iter()
        .chain(framework_crate_dirs())
    {
        let manifest = fs::read_to_string(dir.join("Cargo.toml")).unwrap_or_default();
        let Ok(document) = manifest.parse::<toml::Table>() else {
            continue;
        };
        let Some(toml::Value::Table(features)) = document.get("features") else {
            continue;
        };
        if !features.contains_key("contract") {
            continue;
        }
        let plugin_by_default = match features.get("default") {
            Some(toml::Value::Array(values)) => values
                .iter()
                .any(|value| matches!(value, toml::Value::String(name) if name == "plugin")),
            _ => false,
        };
        found.insert(crate_name(&dir), plugin_by_default);
    }
    found
}

fn composition_calls() -> BTreeSet<String> {
    let mut edges = BTreeSet::new();
    for dir in plugin_crate_dirs()
        .into_iter()
        .chain(framework_crate_dirs())
    {
        let consumer = crate_name(&dir);
        let mut files = Vec::new();
        collect_rs(&dir.join("src"), &mut files);
        for file in files {
            let text = strip(&fs::read_to_string(&file).unwrap_or_default());
            let mut cursor = text.as_str();
            while let Some(index) = cursor.find("add_plugin(nightshade_") {
                let rest = &cursor[index + "add_plugin(".len()..];
                let stem: String = rest
                    .chars()
                    .take_while(|character| character.is_alphanumeric() || *character == '_')
                    .collect();
                let named = stem.replace('_', "-");
                if named != consumer {
                    edges.insert(format!("{consumer} -> {named}"));
                }
                cursor = &rest[stem.len()..];
            }
        }
    }
    edges
}

fn actual_plugin_edges() -> BTreeSet<String> {
    let splitting = splitting_crates();
    let mut edges = BTreeSet::new();
    for dir in plugin_crate_dirs()
        .into_iter()
        .chain(framework_crate_dirs())
    {
        let consumer = crate_name(&dir);
        let manifest = fs::read_to_string(dir.join("Cargo.toml")).unwrap_or_default();
        for entry in manifest_dependencies(&manifest).into_values() {
            let asked = entry.features.contains("plugin");
            let inherited =
                entry.default_features && splitting.get(&entry.name).copied().unwrap_or(false);
            if entry.name.starts_with("nightshade-") && (asked || inherited) {
                edges.insert(format!("{consumer} -> {}", entry.name));
            }
        }
    }
    edges.extend(composition_calls());
    edges
}

fn plugin_edges_off_the_ledger() -> Vec<String> {
    let allowed = recorded_plugin_edges();
    actual_plugin_edges()
        .into_iter()
        .filter(|edge| !allowed.contains(edge))
        .collect()
}

fn stale_plugin_edges() -> Vec<String> {
    let actual = actual_plugin_edges();
    recorded_plugin_edges()
        .into_iter()
        .filter(|edge| !actual.contains(edge))
        .collect()
}

const LEVELS: [&str; 4] = ["describe", "domain", "compose", "draw"];

fn declared_level(dir: &Path) -> Option<String> {
    Some(dir.parent()?.file_name()?.to_string_lossy().to_string())
}

fn plugin_levels() -> BTreeMap<String, usize> {
    let mut levels = BTreeMap::new();
    for dir in plugin_crate_dirs() {
        let Some(level) = declared_level(&dir) else {
            continue;
        };
        let Some(rank) = LEVELS.iter().position(|name| *name == level) else {
            continue;
        };
        levels.insert(crate_name(&dir), rank);
    }
    levels
}

fn plugin_crates_without_a_level() -> Vec<String> {
    let mut reported = Vec::new();
    for entry in fs::read_dir("crates/plugins")
        .into_iter()
        .flatten()
        .flatten()
    {
        let path = entry.path();
        let name = crate_name(&path);
        if path.join("src").is_dir() {
            reported.push(format!("{name} sits at no level"));
        } else if path.is_dir() && !LEVELS.contains(&name.as_str()) {
            reported.push(format!("{name} is not a level"));
        }
    }
    reported.sort();
    reported
}

fn crate_edges_pointing_up() -> Vec<String> {
    let levels = plugin_levels();
    let mut reported = Vec::new();
    for dir in plugin_crate_dirs() {
        let consumer = crate_name(&dir);
        let Some(rank) = levels.get(&consumer).copied() else {
            continue;
        };
        let manifest = fs::read_to_string(dir.join("Cargo.toml")).unwrap_or_default();
        for entry in manifest_dependencies(&manifest).into_values() {
            let Some(other) = levels.get(&entry.name).copied() else {
                continue;
            };
            if other > rank {
                reported.push(format!(
                    "{consumer} ({}) names {} ({})",
                    LEVELS[rank], entry.name, LEVELS[other]
                ));
            }
        }
    }
    reported.sort();
    reported.dedup();
    reported
}

const BACKENDS: [&str; 2] = ["nightshade-vulkan", "nightshade-wgpu"];

fn backends_named_outside_the_drawing_levels() -> Vec<String> {
    let mut dirs = Vec::new();
    for dir in plugin_crate_dirs() {
        let level = declared_level(&dir).unwrap_or_default();
        if level != "compose" && level != "draw" {
            dirs.push(dir);
        }
    }
    dirs.extend(framework_crate_dirs());
    dirs.extend(crate_dirs("crates/support"));
    dirs.extend(crate_dirs("crates"));

    let mut reported = Vec::new();
    for dir in dirs {
        let name = crate_name(&dir);
        let Ok(manifest) = fs::read_to_string(dir.join("Cargo.toml")) else {
            continue;
        };
        let declared = declared_dependencies(&manifest);
        for backend in BACKENDS {
            if declared.contains(backend) {
                reported.push(format!("{name} names {backend}"));
            }
        }
    }
    reported.sort();
    reported.dedup();
    reported
}

fn sibling_plugin_edges() -> Vec<String> {
    let levels = plugin_levels();
    let composition = LEVELS
        .iter()
        .position(|name| *name == "compose")
        .unwrap_or(LEVELS.len() - 1);
    let mut reported = Vec::new();
    for edge in actual_plugin_edges() {
        let Some((consumer, other)) = edge.split_once(" -> ") else {
            continue;
        };
        let (Some(rank), Some(reached)) =
            (levels.get(consumer).copied(), levels.get(other).copied())
        else {
            continue;
        };
        if rank == reached && rank < composition {
            reported.push(format!("{edge} inside {}", LEVELS[rank]));
        }
    }
    reported.sort();
    reported
}

pub(crate) fn collect_rs(dir: &Path, out: &mut Vec<PathBuf>) {
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

fn crate_refs(text: &str) -> Vec<String> {
    let Ok(parsed) = syn::parse_file(text) else {
        return Vec::new();
    };
    let mut visitor = CrateRefVisitor {
        references: Vec::new(),
    };
    syn::visit::Visit::visit_file(&mut visitor, &parsed);
    visitor.references
}

struct CrateRefVisitor {
    references: Vec<String>,
}

impl CrateRefVisitor {
    fn walk_use_tree(&mut self, tree: &syn::UseTree, prefix: Option<String>) {
        match tree {
            syn::UseTree::Path(path) => {
                let name = path.ident.to_string();
                match prefix {
                    None if name == "crate" => self.walk_use_tree(&path.tree, Some(String::new())),
                    None => {}
                    Some(prefix) if prefix.is_empty() => {
                        self.references.push(name.clone());
                        self.walk_use_tree(&path.tree, Some(name));
                    }
                    Some(_) => {}
                }
            }
            syn::UseTree::Group(group) => {
                for item in &group.items {
                    self.walk_use_tree(item, prefix.clone());
                }
            }
            syn::UseTree::Name(name) => {
                if prefix.as_deref() == Some("") {
                    self.references.push(name.ident.to_string());
                }
            }
            syn::UseTree::Rename(rename) => {
                if prefix.as_deref() == Some("") {
                    self.references.push(rename.ident.to_string());
                }
            }
            syn::UseTree::Glob(_) => {}
        }
    }
}

impl<'ast> syn::visit::Visit<'ast> for CrateRefVisitor {
    fn visit_item_use(&mut self, item: &'ast syn::ItemUse) {
        self.walk_use_tree(&item.tree, None);
    }

    fn visit_path(&mut self, path: &'ast syn::Path) {
        let mut segments = path.segments.iter();
        if let Some(first) = segments.next()
            && first.ident == "crate"
            && let Some(second) = segments.next()
        {
            self.references.push(second.ident.to_string());
        }
        syn::visit::visit_path(self, path);
    }
}

pub(crate) fn strip(source: &str) -> String {
    strip_comments(&strip_test_mods(source))
}

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

fn find_mod_brace(after_attr: &str) -> Option<usize> {
    let mod_index = after_attr.find("mod ")?;
    let brace_index = after_attr[mod_index..].find('{')?;
    Some(mod_index + brace_index)
}

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

const BANDS: [&str; 6] = [
    "data",
    "components",
    "resources",
    "queries",
    "commands",
    "systems",
];

fn band_rank(name: &str) -> Option<usize> {
    BANDS.iter().position(|band| *band == name)
}

fn bands_pointing_up() -> Vec<String> {
    let mut reported = Vec::new();
    for dir in crate_dirs("crates")
        .into_iter()
        .chain(plugin_crate_dirs())
        .chain(framework_crate_dirs())
    {
        let name = crate_name(&dir);
        let source = dir.join("src");
        let mut files = Vec::new();
        collect_rust_files(&source, &mut files);
        files.sort();
        for file in &files {
            let Ok(relative) = file.strip_prefix(&source) else {
                continue;
            };
            let Some(first) = relative.components().next() else {
                continue;
            };
            let band = first
                .as_os_str()
                .to_string_lossy()
                .trim_end_matches(".rs")
                .to_string();
            let Some(rank) = band_rank(&band) else {
                continue;
            };
            let text = fs::read_to_string(file).unwrap_or_default();
            for reference in crate_refs(&text) {
                let target = reference.split("::").next().unwrap_or_default();
                let Some(other) = band_rank(target) else {
                    continue;
                };
                if other > rank {
                    reported.push(format!(
                        "{name}: {} names {target}, which is a band above {band}",
                        display_path(file)
                    ));
                }
            }
        }
    }
    reported.sort();
    reported.dedup();
    reported
}

fn unrecorded_inner_cycles() -> Vec<(String, Vec<String>)> {
    let recorded = recorded_plugin_cycles();
    let mut out = Vec::new();
    for dir in crate_dirs("crates")
        .into_iter()
        .chain(plugin_crate_dirs())
        .chain(framework_crate_dirs())
    {
        let plugin = crate_name(&dir);
        let base = dir.join("src");
        let mut inner_modules: BTreeSet<String> = BTreeSet::new();
        if let Ok(entries) = fs::read_dir(&base) {
            for entry in entries.flatten() {
                inner_modules.insert(module_name(&entry.path()));
            }
        }
        inner_modules.remove("lib");
        let mut files = Vec::new();
        collect_rs(&base, &mut files);
        let mut edges: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
        for file in &files {
            let Ok(relative) = file.strip_prefix(&base) else {
                continue;
            };
            let source = relative
                .components()
                .next()
                .map(|part| strip_rs(&part.as_os_str().to_string_lossy()))
                .unwrap_or_default();
            if source == "lib" {
                continue;
            }
            let text = strip(&fs::read_to_string(file).unwrap_or_default());
            for reference in crate_refs(&text) {
                let Some(target) = reference.split("::").next() else {
                    continue;
                };
                if target == source
                    || target == "prelude"
                    || source == "prelude"
                    || !inner_modules.contains(target)
                {
                    continue;
                }
                edges
                    .entry(source.clone())
                    .or_default()
                    .insert(target.to_string());
            }
        }
        for mut component in strongly_connected(&edges) {
            if component.len() < 2 {
                continue;
            }
            component.sort();
            let key = format!("{plugin}: {}", component.join(", "));
            if !recorded.contains(&key) {
                out.push((plugin.clone(), component));
            }
        }
    }
    out
}

fn unclamped_texture_extents() -> Vec<String> {
    const MARKERS: [&str; 2] = [
        ".create_texture(&TextureDescriptor {",
        ".create_texture(&wgpu::TextureDescriptor {",
    ];
    let mut files = Vec::new();
    collect_rs(Path::new(RENDERER), &mut files);
    files.sort();
    let mut out = Vec::new();
    for file in &files {
        let text = fs::read_to_string(file).unwrap_or_default();
        for marker in MARKERS {
            let mut cursor = 0;
            while let Some(found) = text[cursor..].find(marker) {
                let open = cursor + found + marker.len();
                for line in brace_body(&text, open).lines() {
                    let Some(value) = line.trim().strip_prefix("width: ") else {
                        continue;
                    };
                    let Some(value) = value.strip_suffix(',') else {
                        continue;
                    };
                    if !value.chars().all(|character| character.is_ascii_digit()) {
                        out.push(format!("{} allocates from {value}", display_path(file)));
                    }
                }
                cursor = open;
            }
        }
    }
    out
}

fn brace_body(text: &str, open: usize) -> &str {
    let mut depth = 1usize;
    for (offset, character) in text[open..].char_indices() {
        match character {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return &text[open..open + offset];
                }
            }
            _ => {}
        }
    }
    &text[open..]
}

fn recorded_plugin_cycles() -> BTreeSet<String> {
    let path = Path::new("tools/dag-check/plugin-cycles.txt");
    let Ok(text) = fs::read_to_string(path) else {
        return BTreeSet::new();
    };
    text.lines()
        .filter(|line| !line.starts_with('#') && !line.starts_with(' ') && !line.is_empty())
        .map(|line| line.trim().to_string())
        .collect()
}

fn collect_rust_files(dir: &Path, out: &mut Vec<std::path::PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_rust_files(&path, out);
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            out.push(path);
        }
    }
}

fn plugins_declaring_gpu_api() -> Vec<String> {
    const GPU_APIS: [&str; 2] = ["wgpu", "ash"];
    let recorded: BTreeSet<String> = fs::read_to_string("tools/dag-check/gpu-api-declarations.txt")
        .unwrap_or_default()
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(|line| line.to_owned())
        .collect();
    let mut found = Vec::new();
    for dir in plugin_crate_dirs()
        .into_iter()
        .chain(framework_crate_dirs())
        .chain(crate_dirs("crates"))
    {
        let name = crate_name(&dir);
        let manifest = fs::read_to_string(dir.join("Cargo.toml")).unwrap_or_default();
        let declared = manifest_dependencies(&manifest);
        for api in GPU_APIS {
            if declared.contains_key(api) && !recorded.contains(&format!("{name} {api}")) {
                found.push(format!("{name} declares {api}"));
            }
        }
    }
    found.sort();
    found.dedup();
    found
}

fn backend_facade_names(crate_dir: &Path) -> BTreeSet<String> {
    let mut names = BTreeSet::new();
    let Ok(text) = fs::read_to_string(crate_dir.join("src/lib.rs")) else {
        return names;
    };
    let Some(open) = text.find("pub mod backend {") else {
        return names;
    };
    let body = brace_body(&text, open + "pub mod backend ".len());
    let mut cursor = body;
    while let Some(index) = cursor.find("pub mod ") {
        let rest = &cursor[index + "pub mod ".len()..];
        names.insert(
            rest.chars()
                .take_while(|character| character.is_alphanumeric() || *character == '_')
                .collect(),
        );
        cursor = rest;
    }
    let mut cursor = body;
    while let Some(index) = cursor.find("pub use ") {
        let rest = &cursor[index + "pub use ".len()..];
        let end = rest.find(';').unwrap_or(rest.len());
        for piece in rest[..end]
            .replace(['{', '}', '\n'], " ")
            .replace("::", " ")
            .split([',', ' '])
        {
            let piece = piece.trim();
            if !piece.is_empty() {
                names.insert(piece.to_string());
            }
        }
        cursor = &rest[end..];
    }
    names
}

fn names_marker(line: &str, marker: &str) -> bool {
    let mut cursor = line;
    let mut consumed = 0;
    while let Some(at) = cursor.find(marker) {
        let before = line[..consumed + at].chars().next_back();
        if !before.is_some_and(is_ident_char) {
            return true;
        }
        consumed += at + marker.len();
        cursor = &line[consumed..];
    }
    false
}

fn is_ident_char(character: char) -> bool {
    character.is_alphanumeric() || character == '_'
}

fn behavior_in_the_description() -> Vec<String> {
    const MARKERS: [(&str, &str); 5] = [
        ("impl Plugin", "a plugin"),
        ("push_frame_system", "a registered system"),
        ("nightshade::app", "the app"),
        ("wgpu::", "wgpu"),
        ("ash::", "ash"),
    ];
    let mut found = Vec::new();
    let mut files = Vec::new();
    collect_rust_files(Path::new(DESCRIPTION), &mut files);
    files.sort();
    for file in files {
        let installing = file.file_name().is_some_and(|name| name == "plugin.rs");
        let Ok(text) = fs::read_to_string(&file) else {
            continue;
        };
        let text = strip(&text);
        for (index, line) in text.lines().enumerate() {
            for (marker, what) in MARKERS {
                if installing && matches!(marker, "impl Plugin" | "nightshade::app") {
                    continue;
                }
                if names_marker(line, marker) {
                    found.push(format!(
                        "{}:{} names {what}",
                        display_path(&file),
                        index + 1
                    ));
                }
            }
        }
    }
    found
}

fn crates_with_a_facade() -> Vec<(String, PathBuf)> {
    let mut found = Vec::new();
    for dir in plugin_crate_dirs() {
        let Ok(text) = fs::read_to_string(dir.join("src/lib.rs")) else {
            continue;
        };
        if !text.contains("pub mod backend {") {
            continue;
        }
        let Some(name) = dir.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        found.push((name.replace('-', "_"), dir.clone()));
    }
    found.sort();
    found
}

fn backend_paths_outside_the_facade() -> Vec<String> {
    let mut found = Vec::new();
    for (krate, dir) in crates_with_a_facade() {
        let crate_dir = dir.as_path();
        let published = backend_facade_names(crate_dir);
        if published.is_empty() {
            continue;
        }
        let stem = format!("{krate}::backend::");
        let mut dirs = workspace_source_dirs();
        dirs.push(PathBuf::from("tools/shader-check/src"));
        dirs.push(PathBuf::from("tools/dag-check/src"));
        dirs.push(PathBuf::from(
            "crates/plugins/compose/nightshade-ui/examples",
        ));
        for dir in dirs {
            if dir.starts_with(crate_dir) {
                continue;
            }
            let mut files = Vec::new();
            collect_rust_files(&dir, &mut files);
            files.sort();
            for file in files {
                let Ok(text) = fs::read_to_string(&file) else {
                    continue;
                };
                for (index, line) in text.lines().enumerate() {
                    let mut cursor = line;
                    while let Some(at) = cursor.find(&stem) {
                        let rest = &cursor[at + stem.len()..];
                        let name: String = rest
                            .chars()
                            .take_while(|character| {
                                character.is_alphanumeric() || *character == '_'
                            })
                            .collect();
                        if !name.is_empty() && !published.contains(&name) {
                            found.push(format!(
                                "{}:{} names {krate}::backend::{name}",
                                display_path(&file),
                                index + 1
                            ));
                        }
                        cursor = rest;
                    }
                }
            }
        }
    }
    found.sort();
    found.dedup();
    found
}

fn workspace_source_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    for group in [
        "crates",
        "crates/plugins",
        "crates/framework",
        "apps",
        "apps/features",
        "apps/games",
        "apps/ui",
    ] {
        let Ok(entries) = fs::read_dir(group) else {
            continue;
        };
        for entry in entries.flatten() {
            let source = entry.path().join("src");
            if source.is_dir() {
                dirs.push(source);
            }
        }
    }
    for dir in framework_crate_dirs()
        .into_iter()
        .chain(plugin_crate_dirs())
    {
        let source = dir.join("src");
        if source.is_dir() {
            dirs.push(source);
        }
    }
    let template = PathBuf::from("template/src");
    if template.is_dir() {
        dirs.push(template);
    }
    dirs.sort();
    dirs
}

struct WorldBuildVisitor {
    found: Vec<proc_macro2::Span>,
}

impl<'ast> syn::visit::Visit<'ast> for WorldBuildVisitor {
    fn visit_item(&mut self, item: &'ast syn::Item) {
        if item_is_test_gated(item) {
            return;
        }
        syn::visit::visit_item(self, item);
    }

    fn visit_expr_call(&mut self, call: &'ast syn::ExprCall) {
        if let syn::Expr::Path(path) = &*call.func
            && path_builds_a_bare_world(&path.path)
        {
            self.found
                .push(path.path.segments.last().unwrap().ident.span());
        }
        syn::visit::visit_expr_call(self, call);
    }
}

fn item_is_test_gated(item: &syn::Item) -> bool {
    let attributes: &[syn::Attribute] = match item {
        syn::Item::Mod(value) => &value.attrs,
        syn::Item::Fn(value) => &value.attrs,
        syn::Item::Impl(value) => &value.attrs,
        _ => return false,
    };
    attributes.iter().any(|attribute| {
        attribute.path().is_ident("cfg") && attribute.to_token_stream().to_string().contains("test")
    })
}

fn path_builds_a_bare_world(path: &syn::Path) -> bool {
    let mut segments = path.segments.iter().rev();
    let Some(last) = segments.next() else {
        return false;
    };
    if last.ident != "new" && last.ident != "default" {
        return false;
    }
    segments.next().is_some_and(|owner| owner.ident == "World")
}

fn file_holds_tests(file: &Path) -> bool {
    file.file_name().is_some_and(|name| name == "tests.rs")
        || file
            .components()
            .any(|component| component.as_os_str() == "tests")
}

fn untracked_world_builds() -> Vec<String> {
    let mut found = Vec::new();
    for dir in workspace_source_dirs() {
        let mut files = Vec::new();
        collect_rust_files(&dir, &mut files);
        files.sort();
        for file in files {
            if file_holds_tests(&file) {
                continue;
            }
            let Ok(text) = fs::read_to_string(&file) else {
                continue;
            };
            let Ok(parsed) = syn::parse_file(&text) else {
                continue;
            };
            let mut visitor = WorldBuildVisitor { found: Vec::new() };
            syn::visit::Visit::visit_file(&mut visitor, &parsed);
            for span in visitor.found {
                found.push(format!("{}:{}", display_path(&file), span.start().line));
            }
        }
    }
    found
}

fn components_that_are_objects() -> Vec<String> {
    let recorded = recorded_component_sweeps();
    app_component_objects()
        .into_iter()
        .filter(|(app, _)| !recorded.contains(app))
        .map(|(_, entry)| entry)
        .collect()
}

fn stale_component_sweeps() -> Vec<String> {
    let dirty: BTreeSet<String> = app_component_objects()
        .into_iter()
        .map(|(app, _)| app)
        .collect();
    recorded_component_sweeps()
        .into_iter()
        .filter(|app| !dirty.contains(app))
        .collect()
}

const RAW_INPUT_READS: [&str; 8] = [
    "is_key_pressed(",
    "keyboard.just_pressed(",
    "keyboard.just_released(",
    "just_pressed_keys",
    "pad.pressed(",
    "pad.just_pressed(",
    "pad.axis(",
    "Res<Gamepad>",
];

fn raw_input_reads_in_games() -> Vec<String> {
    let recorded = recorded_raw_input();
    game_raw_input_reads()
        .into_iter()
        .filter(|(app, _)| !recorded.contains(app))
        .map(|(_, entry)| entry)
        .collect()
}

fn stale_raw_input_records() -> Vec<String> {
    let dirty: BTreeSet<String> = game_raw_input_reads()
        .into_iter()
        .map(|(app, _)| app)
        .collect();
    recorded_raw_input()
        .into_iter()
        .filter(|app| !dirty.contains(app))
        .collect()
}

fn recorded_raw_input() -> BTreeSet<String> {
    let path = Path::new("tools/dag-check/raw-input.txt");
    let Ok(text) = fs::read_to_string(path) else {
        return BTreeSet::new();
    };
    text.lines()
        .filter(|line| !line.starts_with('#') && !line.trim().is_empty())
        .map(|line| line.trim().to_string())
        .collect()
}

fn game_raw_input_reads() -> Vec<(String, String)> {
    let mut found = Vec::new();
    let mut roots = Vec::new();
    for (group, prefix) in [
        ("apps/games", ""),
        ("apps/games/multiplayer", "multiplayer/"),
    ] {
        let Ok(entries) = fs::read_dir(group) else {
            continue;
        };
        for entry in entries.flatten() {
            let source = entry.path().join("src");
            if !source.is_dir() {
                continue;
            }
            let app = format!("{prefix}{}", entry.file_name().to_string_lossy());
            roots.push((app.clone(), source));
            let Ok(crates) = fs::read_dir(entry.path().join("crates")) else {
                continue;
            };
            for member in crates.flatten() {
                let source = member.path().join("src");
                if source.is_dir() {
                    roots.push((app.clone(), source));
                }
            }
        }
    }
    roots.push(("template".to_string(), PathBuf::from("template/src")));
    roots.sort();
    for (app, root) in roots {
        let mut files = Vec::new();
        collect_rust_files(&root, &mut files);
        files.sort();
        for file in files {
            let Ok(text) = fs::read_to_string(&file) else {
                continue;
            };
            let flat: String = text.split_whitespace().collect();
            if RAW_INPUT_READS.iter().any(|needle| flat.contains(needle)) {
                found.push((app.clone(), format!("{app}: {}", file.display())));
            }
        }
    }
    found
}

fn recorded_component_sweeps() -> BTreeSet<String> {
    let path = Path::new("tools/dag-check/component-sweep.txt");
    let Ok(text) = fs::read_to_string(path) else {
        return BTreeSet::new();
    };
    text.lines()
        .filter(|line| !line.starts_with('#') && !line.trim().is_empty())
        .map(|line| line.trim().to_string())
        .collect()
}

fn app_component_objects() -> Vec<(String, String)> {
    let mut found = Vec::new();
    let mut files = Vec::new();
    collect_rust_files(Path::new("apps"), &mut files);
    collect_rust_files(Path::new("template/src"), &mut files);
    files.sort();

    for file in files {
        if !is_components_band(&file) {
            continue;
        }
        let Ok(text) = fs::read_to_string(&file) else {
            continue;
        };
        let app = app_name(&file);
        for (name, line, fields) in declared_structs(&text) {
            if fields == 1 {
                continue;
            }
            let shape = if fields == 0 {
                "an empty marker"
            } else {
                "two or more values"
            };
            found.push((
                app.clone(),
                format!("{}:{} {name} is {shape}", display_path(&file), line),
            ));
        }
    }
    found
}

fn is_components_band(file: &Path) -> bool {
    let path = file.to_string_lossy().replace('\\', "/");
    path.ends_with("/components.rs") || path.contains("/components/")
}

fn app_name(file: &Path) -> String {
    let path = file.to_string_lossy().replace('\\', "/");
    let parts: Vec<&str> = path.split('/').collect();
    match parts.iter().position(|part| *part == "src") {
        Some(0) | None => "template".to_string(),
        Some(index) => parts[index - 1].to_string(),
    }
}

fn declared_structs(text: &str) -> Vec<(String, usize, usize)> {
    let Ok(parsed) = syn::parse_file(text) else {
        return Vec::new();
    };
    let mut found = Vec::new();
    collect_structs(&parsed.items, &mut found);
    found
}

fn collect_structs(items: &[syn::Item], found: &mut Vec<(String, usize, usize)>) {
    for item in items {
        match item {
            syn::Item::Mod(module) => {
                if let Some((_, inner)) = &module.content {
                    collect_structs(inner, found);
                }
            }
            syn::Item::Struct(declared) => {
                if !matches!(declared.vis, syn::Visibility::Public(_)) {
                    continue;
                }
                let fields = match &declared.fields {
                    syn::Fields::Named(named) => named.named.len(),
                    syn::Fields::Unnamed(unnamed) => {
                        let types: BTreeSet<String> = unnamed
                            .unnamed
                            .iter()
                            .map(|field| field.ty.to_token_stream().to_string())
                            .collect();
                        if types.len() == 1 {
                            1
                        } else {
                            unnamed.unnamed.len()
                        }
                    }
                    syn::Fields::Unit => 0,
                };
                found.push((
                    declared.ident.to_string(),
                    declared.ident.span().start().line,
                    fields,
                ));
            }
            _ => {}
        }
    }
}
