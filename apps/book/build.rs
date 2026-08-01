use std::fmt::Write as _;
use std::path::PathBuf;

fn main() {
    let manifest = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect("manifest directory"));
    let book = manifest.join("book");
    let summary = book.join("SUMMARY.md");

    println!("cargo:rerun-if-changed={}", book.display());
    println!("cargo:rerun-if-changed={}", summary.display());

    let text = std::fs::read_to_string(&summary).expect("the summary of the book");
    let entries = read_summary(&text);

    let mut generated = String::new();
    generated.push_str("pub const CHAPTERS: &[ChapterSource] = &[\n");
    for entry in &entries {
        let source = book.join(&entry.path);
        println!("cargo:rerun-if-changed={}", source.display());
        let include = source.to_string_lossy().replace('\\', "/");
        let part = match &entry.part {
            Some(part) => format!("Some({part:?})"),
            None => "None".to_string(),
        };
        let title = &entry.title;
        let depth = entry.depth;
        let path = entry.path.replace('\\', "/");
        let _ = writeln!(
            generated,
            "    ChapterSource {{ part: {part}, title: {title:?}, depth: {depth}, path: {path:?}, markdown: include_str!({include:?}) }},"
        );
    }
    generated.push_str("];\n");

    let out = PathBuf::from(std::env::var("OUT_DIR").expect("output directory"));
    std::fs::write(out.join("chapters.rs"), generated).expect("the generated chapter table");

    write_image_table(&manifest, &out);
    write_source_table(&manifest, &out);
}

fn write_source_table(manifest: &std::path::Path, out: &std::path::Path) {
    const SHARED: [&str; 3] = [
        "src/data/demo.rs",
        "src/commands/scene.rs",
        "src/systems/demos.rs",
    ];

    let demos = manifest.join("src/systems/demos");
    println!("cargo:rerun-if-changed={}", demos.display());

    let mut names: Vec<String> = std::fs::read_dir(&demos)
        .map(|entries| {
            entries
                .filter_map(|entry| entry.ok())
                .map(|entry| entry.path())
                .filter(|path| path.extension().is_some_and(|kind| kind == "rs"))
                .filter_map(|path| {
                    path.file_stem()
                        .map(|stem| stem.to_string_lossy().to_string())
                })
                .collect()
        })
        .unwrap_or_default();
    names.sort();

    let mut generated = String::new();

    let mut shared: Vec<PathBuf> = SHARED.iter().map(|file| manifest.join(file)).collect();
    shared.sort();
    generated.push_str(
        "pub const SHARED_SOURCES: &[SourceFile] = &[
",
    );
    for file in &shared {
        write_source_file(&mut generated, manifest, file, false);
    }
    generated.push_str(
        "];

",
    );

    generated.push_str(
        "pub const DEMO_SOURCES: &[DemoSource] = &[
",
    );
    for name in &names {
        let mut own = vec![demos.join(format!("{name}.rs"))];
        gather_rust(&demos.join(name), &mut own);
        own.sort();

        let _ = writeln!(generated, "    DemoSource {{ demo: {name:?}, own: &[");
        for file in &own {
            write_source_file(&mut generated, manifest, file, true);
        }
        generated.push_str(
            "    ] },
",
        );
    }
    generated.push_str(
        "];
",
    );

    std::fs::write(out.join("sources.rs"), generated).expect("the generated source table");
}

fn gather_rust(folder: &std::path::Path, found: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(folder) else {
        return;
    };
    for entry in entries.filter_map(|entry| entry.ok()) {
        let path = entry.path();
        if path.is_dir() {
            gather_rust(&path, found);
        } else if path.extension().is_some_and(|kind| kind == "rs") {
            found.push(path);
        }
    }
}

fn write_source_file(
    generated: &mut String,
    manifest: &std::path::Path,
    file: &std::path::Path,
    own: bool,
) {
    println!("cargo:rerun-if-changed={}", file.display());
    let path = file
        .strip_prefix(manifest)
        .unwrap_or(file)
        .to_string_lossy()
        .replace('\\', "/");
    let include = file.to_string_lossy().replace('\\', "/");
    let _ = writeln!(
        generated,
        "        SourceFile {{ path: {path:?}, own: {own}, text: include_str!({include:?}) }},"
    );
}

fn write_image_table(manifest: &std::path::Path, out: &std::path::Path) {
    let images = manifest.join("images");
    println!("cargo:rerun-if-changed={}", images.display());

    let mut names: Vec<PathBuf> = std::fs::read_dir(&images)
        .map(|entries| {
            entries
                .filter_map(|entry| entry.ok())
                .map(|entry| entry.path())
                .collect()
        })
        .unwrap_or_default();
    names.sort();

    let mut generated = String::new();
    generated.push_str(
        "pub const IMAGES: &[ImageSource] = &[
",
    );
    for path in &names {
        let Some(stem) = path.file_stem().and_then(|stem| stem.to_str()) else {
            continue;
        };
        println!("cargo:rerun-if-changed={}", path.display());
        let include = path
            .to_string_lossy()
            .replace(std::path::MAIN_SEPARATOR, "/");
        let _ = writeln!(
            generated,
            "    ImageSource {{ name: {stem:?}, bytes: include_bytes!({include:?}) }},"
        );
    }
    generated.push_str(
        "];
",
    );
    std::fs::write(out.join("images.rs"), generated).expect("the generated image table");
}

struct SummaryEntry {
    part: Option<String>,
    title: String,
    path: String,
    depth: usize,
}

fn read_summary(text: &str) -> Vec<SummaryEntry> {
    let mut entries = Vec::new();
    let mut part: Option<String> = None;
    let mut pending_part: Option<String> = None;

    for line in text.lines() {
        let trimmed = line.trim_start();
        if let Some(heading) = trimmed.strip_prefix("# ") {
            let heading = heading.trim();
            if !heading.eq_ignore_ascii_case("summary") {
                pending_part = Some(heading.to_string());
            }
            continue;
        }

        let Some((title, path)) = read_link(trimmed) else {
            continue;
        };
        if !path.ends_with(".md") {
            continue;
        }

        if let Some(next) = pending_part.take() {
            part = Some(next);
        }

        let indent = line.len() - trimmed.len();
        entries.push(SummaryEntry {
            part: part.clone(),
            title,
            path,
            depth: indent / 2,
        });
    }

    entries
}

fn read_link(line: &str) -> Option<(String, String)> {
    let start = line.find('[')?;
    let end = line[start..].find("](")? + start;
    let close = line[end..].find(')')? + end;
    let title = line[start + 1..end].trim().to_string();
    let path = line[end + 2..close].trim().to_string();
    (!title.is_empty() && !path.is_empty()).then_some((title, path))
}
