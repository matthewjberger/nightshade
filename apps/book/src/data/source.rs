pub struct SourceFile {
    pub path: &'static str,
    pub own: bool,
    pub text: &'static str,
}

pub struct DemoSource {
    pub demo: &'static str,
    pub own: &'static [SourceFile],
}

include!(concat!(env!("OUT_DIR"), "/sources.rs"));

pub fn files_of(demo: &str) -> Vec<&'static SourceFile> {
    let own = DEMO_SOURCES
        .iter()
        .find(|source| source.demo == demo)
        .map(|source| source.own)
        .unwrap_or(&[]);
    if own.is_empty() {
        return Vec::new();
    }

    let mut files: Vec<&'static SourceFile> = own.iter().chain(SHARED_SOURCES).collect();
    files.sort_by_key(|file| file.path);
    files
}

pub struct TreeLine {
    pub depth: usize,
    pub name: String,
    pub file: Option<usize>,
    pub own: bool,
}

pub fn tree_of(files: &[&'static SourceFile]) -> Vec<TreeLine> {
    let mut lines: Vec<TreeLine> = Vec::new();
    let mut open: Vec<&str> = Vec::new();

    for (index, file) in files.iter().enumerate() {
        let parts: Vec<&str> = file.path.split('/').collect();
        let Some((name, folders)) = parts.split_last() else {
            continue;
        };

        let shared = folders
            .iter()
            .zip(open.iter())
            .take_while(|(folder, held)| folder == held)
            .count();
        open.truncate(shared);

        for folder in &folders[shared..] {
            lines.push(TreeLine {
                depth: open.len(),
                name: (*folder).to_string(),
                file: None,
                own: false,
            });
            open.push(folder);
        }

        lines.push(TreeLine {
            depth: open.len(),
            name: (*name).to_string(),
            file: Some(index),
            own: file.own,
        });
    }

    lines
}
