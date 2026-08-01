use std::fs;
use std::path::Path;

const DOCUMENT: &str = "CLAUDE.md";

const UNITS: [(&str, usize); 20] = [
    ("zero", 0),
    ("one", 1),
    ("two", 2),
    ("three", 3),
    ("four", 4),
    ("five", 5),
    ("six", 6),
    ("seven", 7),
    ("eight", 8),
    ("nine", 9),
    ("ten", 10),
    ("eleven", 11),
    ("twelve", 12),
    ("thirteen", 13),
    ("fourteen", 14),
    ("fifteen", 15),
    ("sixteen", 16),
    ("seventeen", 17),
    ("eighteen", 18),
    ("nineteen", 19),
];

const TENS: [(&str, usize); 8] = [
    ("twenty", 20),
    ("thirty", 30),
    ("forty", 40),
    ("fifty", 50),
    ("sixty", 60),
    ("seventy", 70),
    ("eighty", 80),
    ("ninety", 90),
];

fn word_value(word: &str) -> Option<usize> {
    let lowered = word.to_lowercase();
    if let Ok(digits) = lowered.parse::<usize>() {
        return Some(digits);
    }
    if let Some((_, value)) = UNITS.iter().find(|(name, _)| *name == lowered) {
        return Some(*value);
    }
    if let Some((_, value)) = TENS.iter().find(|(name, _)| *name == lowered) {
        return Some(*value);
    }
    let (tens, units) = lowered.split_once('-')?;
    let (_, tens_value) = TENS.iter().find(|(name, _)| *name == tens)?;
    let (_, units_value) = UNITS.iter().find(|(name, _)| *name == units)?;
    Some(tens_value + units_value)
}

fn spelled(value: usize) -> String {
    if let Some((name, _)) = UNITS.iter().find(|(_, known)| *known == value) {
        return (*name).to_string();
    }
    if let Some((name, _)) = TENS.iter().find(|(_, known)| *known == value) {
        return (*name).to_string();
    }
    let tens = value / 10 * 10;
    let units = value % 10;
    match (
        TENS.iter().find(|(_, known)| *known == tens),
        UNITS.iter().find(|(_, known)| *known == units),
    ) {
        (Some((tens_name, _)), Some((units_name, _))) => format!("{tens_name}-{units_name}"),
        _ => value.to_string(),
    }
}

fn stated(text: &str, phrase: &str) -> Option<usize> {
    let at = text.find(phrase)?;
    let before = text[..at].trim_end();
    let word = before
        .rsplit(|letter: char| !(letter.is_ascii_alphanumeric() || letter == '-'))
        .find(|piece| !piece.is_empty())?;
    word_value(word)
}

fn directories(path: &str) -> usize {
    fs::read_dir(path)
        .into_iter()
        .flatten()
        .flatten()
        .filter(|entry| entry.path().is_dir())
        .count()
}

fn crates_under(path: &str) -> usize {
    fs::read_dir(path)
        .into_iter()
        .flatten()
        .flatten()
        .filter(|entry| entry.path().is_dir())
        .map(|entry| directories(&entry.path().to_string_lossy()))
        .sum()
}

fn occurrences(path: &str, needle: &str) -> usize {
    fs::read_to_string(path)
        .unwrap_or_default()
        .matches(needle)
        .count()
}

fn manifests_saying(root: &str, needle: &str) -> usize {
    let mut total = 0;
    for level in fs::read_dir(root).into_iter().flatten().flatten() {
        if !level.path().is_dir() {
            continue;
        }
        for crate_dir in fs::read_dir(level.path()).into_iter().flatten().flatten() {
            let manifest = crate_dir.path().join("Cargo.toml");
            if manifest.is_file() {
                total += occurrences(&manifest.to_string_lossy(), needle);
            }
        }
    }
    total
}

fn facts() -> Vec<(&'static str, usize)> {
    vec![
        (
            " plugin crates, each carrying a plugin",
            crates_under("crates/plugins"),
        ),
        (
            " levels a crate may only name downward",
            directories("crates/plugins"),
        ),
        (
            " roots that run a loop or own a backend",
            directories("crates/plugins/compose"),
        ),
        (
            " tools sit at `crates/support/`",
            directories("crates/support"),
        ),
        (
            " crates and every one has a reader",
            crates_under("crates/framework"),
        ),
        (" groups run `base`", directories("crates/framework")),
        (
            " exist, and every one is a contract",
            manifests_saying("crates/plugins", "features = [\"contract\"]"),
        ),
        (
            " render resources and registers the",
            occurrences(
                "crates/plugins/describe/nightshade-render/src/commands/install.rs",
                "hold::<",
            ),
        ),
        (
            " frame systems over them",
            occurrences(
                "crates/plugins/domain/nightshade-graphics/src/plugin.rs",
                "push_frame_system(",
            ),
        ),
    ]
}

pub fn numbers_the_document_gets_wrong() -> Vec<String> {
    let Ok(text) = fs::read_to_string(Path::new(DOCUMENT)) else {
        return vec![format!("{DOCUMENT} is not there to read")];
    };
    let mut reported = Vec::new();
    for (phrase, found) in facts() {
        match stated(&text, phrase) {
            Some(said) if said == found => {}
            Some(said) => reported.push(format!(
                "\"{}\": the file says {}, the tree holds {}",
                phrase.trim(),
                spelled(said),
                spelled(found)
            )),
            None => reported.push(format!(
                "\"{}\": the file states no number there, and the tree holds {}",
                phrase.trim(),
                spelled(found)
            )),
        }
    }
    reported
}
