use crate::data::Token;

pub fn scan(source: &str) -> Result<Vec<Token>, String> {
    let mut tokens = Vec::new();
    let mut indents = vec![0usize];
    let mut depth = 0usize;

    for (number, raw) in source.lines().enumerate() {
        let line = strip_comment(raw);
        if line.trim().is_empty() {
            continue;
        }

        let joined = depth > 0 || ends_open(&tokens);
        if !joined {
            let width = indent_width(line);
            close_or_open(&mut tokens, &mut indents, width);
        }

        scan_line(line.trim(), &mut tokens, &mut depth, number + 1)?;

        if depth == 0 && !ends_open(&tokens) {
            tokens.push(Token::Newline);
        }
    }

    while indents.len() > 1 {
        indents.pop();
        tokens.push(Token::End);
    }
    Ok(tokens)
}

fn strip_comment(line: &str) -> &str {
    match line.find("--") {
        Some(at) => &line[..at],
        None => line,
    }
}

fn indent_width(line: &str) -> usize {
    let mut width = 0;
    for letter in line.chars() {
        match letter {
            ' ' => width += 1,
            '\t' => width += 4,
            _ => break,
        }
    }
    width
}

const CARRIES: &[&str] = &[
    ",", "+", "-", "*", "/", "<", ">", "=", "|", ":", ".", ":=", "+=", "<=", ">=", "!=", "->", "..",
];

fn ends_open(tokens: &[Token]) -> bool {
    matches!(tokens.last(), Some(Token::Symbol(symbol)) if CARRIES.contains(&symbol.as_str()))
}

fn close_or_open(tokens: &mut Vec<Token>, indents: &mut Vec<usize>, width: usize) {
    let current = *indents.last().unwrap_or(&0);
    if width > current {
        indents.push(width);
        tokens.push(Token::Begin);
        return;
    }
    while width < *indents.last().unwrap_or(&0) {
        indents.pop();
        tokens.push(Token::End);
    }
}

fn scan_line(
    line: &str,
    tokens: &mut Vec<Token>,
    depth: &mut usize,
    number: usize,
) -> Result<(), String> {
    let letters: Vec<char> = line.chars().collect();
    let mut index = 0;

    while index < letters.len() {
        let letter = letters[index];

        if letter.is_whitespace() {
            index += 1;
            continue;
        }

        if letter == '"' {
            let mut text = String::new();
            index += 1;
            while index < letters.len() && letters[index] != '"' {
                text.push(letters[index]);
                index += 1;
            }
            if index >= letters.len() {
                return Err(format!("line {number}: a text value has no closing quote"));
            }
            index += 1;
            tokens.push(Token::Text(text));
            continue;
        }

        if letter.is_ascii_digit() {
            let start = index;
            while index < letters.len() && letters[index].is_ascii_digit() {
                index += 1;
            }
            if index + 1 < letters.len() && letters[index] == '.' && letters[index + 1] != '.' {
                index += 1;
                while index < letters.len() && letters[index].is_ascii_digit() {
                    index += 1;
                }
            }
            let text: String = letters[start..index].iter().collect();
            let value = text
                .parse::<f32>()
                .map_err(|_| format!("line {number}: {text} is not a number"))?;
            tokens.push(Token::Number(value));
            continue;
        }

        if letter.is_alphabetic() || letter == '_' {
            let start = index;
            while index < letters.len()
                && (letters[index].is_alphanumeric() || letters[index] == '_')
            {
                index += 1;
            }
            tokens.push(Token::Word(letters[start..index].iter().collect()));
            continue;
        }

        let pair: String = letters[index..(index + 2).min(letters.len())]
            .iter()
            .collect();
        if pair == ":="
            || pair == "+="
            || pair == ".."
            || pair == "<="
            || pair == ">="
            || pair == "!="
            || pair == "->"
        {
            tokens.push(Token::Symbol(pair));
            index += 2;
            continue;
        }

        if "([{".contains(letter) {
            *depth += 1;
        }
        if ")]}".contains(letter) {
            *depth = depth.saturating_sub(1);
        }

        if "()[]{},.:+-*/<>=|".contains(letter) {
            tokens.push(Token::Symbol(letter.to_string()));
            index += 1;
            continue;
        }

        return Err(format!("line {number}: {letter} has no meaning here"));
    }
    Ok(())
}
