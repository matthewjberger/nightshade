use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};

fn main() {
    let manifest = PathBuf::from(
        std::env::var("CARGO_MANIFEST_DIR").expect("cargo names the manifest folder"),
    );
    let root = std::path::absolute(manifest.join("../../../../assets/kits/models"))
        .expect("the kit models folder has an absolute path");
    println!("cargo:rerun-if-changed={}", root.display());

    let mut kits: Vec<(String, PathBuf)> = fs::read_dir(&root)
        .expect("the kit models folder can be read")
        .filter_map(Result::ok)
        .filter(|entry| entry.path().is_dir())
        .map(|entry| {
            (
                entry.file_name().to_string_lossy().into_owned(),
                entry.path(),
            )
        })
        .collect();
    kits.sort_by(|left, right| left.0.cmp(&right.0));

    let mut source = String::new();
    source.push_str("#[derive(Clone, Copy, PartialEq, Eq, Debug)]\npub enum Kit {\n");
    for (name, _) in &kits {
        let _ = writeln!(source, "    {},", variant(name));
    }
    source.push_str("}\n\n");
    source.push_str(
        "pub struct ModelFile {\n    pub name: &'static str,\n    pub kit: Kit,\n    pub bytes: &'static [u8],\n}\n\n",
    );

    let mut models = Vec::new();
    let mut textures = Vec::new();
    for (name, folder) in &kits {
        println!("cargo:rerun-if-changed={}", folder.display());
        let feature = format!("CARGO_FEATURE_{}", name.to_uppercase().replace('-', "_"));
        let enabled = std::env::var_os(feature).is_some();
        if !enabled {
            textures.push(String::from("&[]"));
            continue;
        }
        let mut files: Vec<PathBuf> = fs::read_dir(folder)
            .expect("the kit folder can be read")
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .filter(|path| path.extension().is_some_and(|extension| extension == "glb"))
            .collect();
        files.sort();
        for file in files {
            let model = file
                .file_stem()
                .expect("a model file has a name")
                .to_string_lossy()
                .into_owned();
            models.push(format!(
                "    ModelFile {{\n        name: {model:?},\n        kit: Kit::{},\n        bytes: include_bytes!({}),\n    }},\n",
                variant(name),
                literal(&file)
            ));
        }
        let mut pictures: BTreeMap<String, PathBuf> = BTreeMap::new();
        gather_pictures(folder, &mut pictures);
        let mut entries = String::from("&[\n");
        for (picture, path) in pictures {
            let _ = writeln!(
                entries,
                "        (\"Textures/{picture}\", include_bytes!({})),",
                literal(&path)
            );
        }
        entries.push_str("    ]");
        textures.push(entries);
    }

    let _ = writeln!(source, "pub const KIT_COUNT: usize = {};", kits.len());
    let _ = writeln!(source, "pub const MODEL_COUNT: usize = {};\n", models.len());
    source.push_str("pub static MODELS: [ModelFile; MODEL_COUNT] = [\n");
    for model in &models {
        source.push_str(model);
    }
    source.push_str("];\n\n");
    source.push_str("pub static KIT_TEXTURES: [&[(&str, &[u8])]; KIT_COUNT] = [\n");
    for entry in &textures {
        let _ = writeln!(source, "    {entry},");
    }
    source.push_str("];\n");

    let out = PathBuf::from(std::env::var("OUT_DIR").expect("cargo names the output folder"));
    fs::write(out.join("models.rs"), source).expect("the catalog can be written");
}

fn gather_pictures(folder: &Path, pictures: &mut BTreeMap<String, PathBuf>) {
    let Ok(entries) = fs::read_dir(folder) else {
        return;
    };
    for entry in entries.filter_map(Result::ok) {
        let path = entry.path();
        if path.is_dir() {
            gather_pictures(&path, pictures);
        } else if path.extension().is_some_and(|extension| extension == "png") {
            let picture = path
                .file_name()
                .expect("a picture has a name")
                .to_string_lossy()
                .into_owned();
            pictures.insert(picture, path);
        }
    }
}

fn variant(folder: &str) -> String {
    let mut characters = folder.chars();
    match characters.next() {
        Some(first) => first.to_uppercase().collect::<String>() + characters.as_str(),
        None => String::new(),
    }
}

fn literal(path: &Path) -> String {
    format!("{:?}", path.to_string_lossy().replace('\\', "/"))
}
