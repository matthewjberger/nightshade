use crate::data::screen::Screen;

pub fn is_pregame(screen: Screen) -> bool {
    matches!(
        screen,
        Screen::Setup | Screen::Title | Screen::NewWorld | Screen::Loading
    )
}

pub fn page_the_speech(speech: &str, characters: usize, lines: usize) -> Vec<String> {
    let mut pages: Vec<String> = Vec::new();
    let mut page: Vec<String> = Vec::new();
    let mut used = 0usize;
    for paragraph in speech.split(
        "

",
    ) {
        let wrapped = wrap_the_line(paragraph, characters);
        let wanted = wrapped.len() + usize::from(!page.is_empty());
        if used + wanted > lines && !page.is_empty() {
            pages.push(page.join(
                "
",
            ));
            page.clear();
            used = 0;
        }
        if !page.is_empty() {
            page.push(String::new());
            used += 1;
        }
        for line in wrapped {
            if used == lines && !page.is_empty() {
                pages.push(page.join(
                    "
",
                ));
                page.clear();
                used = 0;
            }
            page.push(line);
            used += 1;
        }
    }
    if !page.is_empty() {
        pages.push(page.join(
            "
",
        ));
    }
    if pages.is_empty() {
        pages.push(String::new());
    }
    pages
}

pub fn wrap_the_line(paragraph: &str, characters: usize) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    let mut line = String::new();
    for word in paragraph.split_whitespace() {
        let wanted = line.chars().count() + usize::from(!line.is_empty()) + word.chars().count();
        if wanted > characters && !line.is_empty() {
            lines.push(std::mem::take(&mut line));
        }
        if !line.is_empty() {
            line.push(' ');
        }
        line.push_str(word);
    }
    if !line.is_empty() {
        lines.push(line);
    }
    lines
}
