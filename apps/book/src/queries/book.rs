use crate::resources::Library;

pub fn previous_page(library: &Library, current: usize) -> Option<usize> {
    (current > 0 && current < library.chapters.len()).then(|| current - 1)
}

pub fn next_page(library: &Library, current: usize) -> Option<usize> {
    (current + 1 < library.chapters.len()).then(|| current + 1)
}

pub fn page_title(library: &Library, index: usize) -> &str {
    library
        .chapters
        .get(index)
        .map(|chapter| chapter.title)
        .unwrap_or("")
}

pub fn breadcrumb(library: &Library, index: usize) -> String {
    let Some(chapter) = library.chapters.get(index) else {
        return String::new();
    };
    match chapter.part {
        Some(part) => format!("{part}  /  {}", chapter.title),
        None => chapter.title.to_string(),
    }
}

pub fn link_target(
    library: &Library,
    from: usize,
    target: &str,
) -> Option<(usize, Option<String>)> {
    if target.starts_with("http") || target.starts_with("mailto:") {
        return None;
    }

    let (path, anchor) = match target.split_once('#') {
        Some((path, anchor)) => (path, Some(anchor.to_string())),
        None => (target, None),
    };

    if path.is_empty() {
        return Some((from, anchor));
    }

    let here = library.chapters.get(from).map(|chapter| chapter.path)?;
    let wanted = resolve(here, path);
    let page = library
        .chapters
        .iter()
        .position(|chapter| chapter.path == wanted)?;
    Some((page, anchor))
}

fn resolve(from: &str, target: &str) -> String {
    let mut parts: Vec<&str> = match from.rsplit_once('/') {
        Some((directory, _)) => directory.split('/').collect(),
        None => Vec::new(),
    };
    if target.starts_with('/') {
        parts.clear();
    }

    for step in target.trim_start_matches('/').split('/') {
        match step {
            "" | "." => {}
            ".." => {
                parts.pop();
            }
            name => parts.push(name),
        }
    }
    parts.join("/")
}

pub fn requested_page(library: &Library, wanted: Option<&str>) -> Option<usize> {
    let wanted = wanted?;
    if let Ok(index) = wanted.parse::<usize>() {
        return (index < library.chapters.len()).then_some(index);
    }
    library
        .chapters
        .iter()
        .position(|chapter| chapter.title.eq_ignore_ascii_case(wanted))
}
