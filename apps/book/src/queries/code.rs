#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Token {
    Plain,
    Keyword,
    Type,
    Text,
    Number,
    Comment,
    Attribute,
}

const KEYWORDS: &[&str] = &[
    "as", "async", "await", "break", "const", "continue", "crate", "dyn", "else", "enum", "extern",
    "false", "fn", "for", "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut", "pub",
    "ref", "return", "self", "Self", "static", "struct", "super", "trait", "true", "type",
    "unsafe", "use", "where", "while",
];

pub fn language_of(path: &str) -> &'static str {
    if path.ends_with(".rs") {
        "rust"
    } else if path.ends_with(".toml") {
        "toml"
    } else if path.ends_with(".wgsl") || path.ends_with(".glsl") {
        "wgsl"
    } else {
        ""
    }
}

pub fn tokenize(line: &str, language: &str) -> Vec<(String, Token)> {
    let rust = matches!(language, "" | "rust" | "rs");
    let slash_comment = rust || matches!(language, "wgsl" | "glsl" | "c" | "cpp" | "js" | "ts");
    let hash_comment = matches!(language, "toml" | "bash" | "sh" | "shell" | "yaml" | "ini");
    let characters: Vec<char> = line.chars().collect();
    let mut pieces: Vec<(String, Token)> = Vec::new();
    let mut index = 0;

    while index < characters.len() {
        let character = characters[index];

        if slash_comment && character == '/' && characters.get(index + 1) == Some(&'/') {
            pieces.push((characters[index..].iter().collect(), Token::Comment));
            break;
        }

        if hash_comment && character == '#' {
            pieces.push((characters[index..].iter().collect(), Token::Comment));
            break;
        }

        if character == '"' {
            let start = index;
            index += 1;
            while index < characters.len() {
                if characters[index] == '\\' {
                    index += 2;
                    continue;
                }
                if characters[index] == '"' {
                    index += 1;
                    break;
                }
                index += 1;
            }
            let end = index.min(characters.len());
            pieces.push((characters[start..end].iter().collect(), Token::Text));
            continue;
        }

        if rust && character == '#' && characters.get(index + 1) == Some(&'[') {
            let start = index;
            while index < characters.len() && characters[index] != ']' {
                index += 1;
            }
            index = (index + 1).min(characters.len());
            pieces.push((characters[start..index].iter().collect(), Token::Attribute));
            continue;
        }

        if character.is_alphanumeric() || character == '_' {
            let start = index;
            while index < characters.len()
                && (characters[index].is_alphanumeric() || characters[index] == '_')
            {
                index += 1;
            }
            let word: String = characters[start..index].iter().collect();
            let token = word_token(&word, rust);
            pieces.push((word, token));
            continue;
        }

        let start = index;
        while index < characters.len()
            && !characters[index].is_alphanumeric()
            && characters[index] != '_'
            && characters[index] != '"'
            && characters[index] != '#'
            && !(characters[index] == '/' && characters.get(index + 1) == Some(&'/'))
        {
            index += 1;
        }
        if index == start {
            index += 1;
        }
        pieces.push((characters[start..index].iter().collect(), Token::Plain));
    }

    merge_runs(pieces)
}

fn merge_runs(pieces: Vec<(String, Token)>) -> Vec<(String, Token)> {
    let mut merged: Vec<(String, Token)> = Vec::with_capacity(pieces.len());
    for (text, token) in pieces {
        match merged.last_mut() {
            Some((held, last)) if *last == token => held.push_str(&text),
            _ => merged.push((text, token)),
        }
    }
    merged
}

fn word_token(word: &str, rust: bool) -> Token {
    if rust && KEYWORDS.contains(&word) {
        return Token::Keyword;
    }
    if word
        .chars()
        .next()
        .is_some_and(|first| first.is_ascii_digit())
    {
        return Token::Number;
    }
    if rust && word.chars().next().is_some_and(char::is_uppercase) {
        return Token::Type;
    }
    Token::Plain
}
