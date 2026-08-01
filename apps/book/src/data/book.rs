pub const CAPTURE_FRAME: u64 = 60;

pub struct ChapterSource {
    pub part: Option<&'static str>,
    pub title: &'static str,
    pub depth: usize,
    pub path: &'static str,
    pub markdown: &'static str,
}

include!(concat!(env!("OUT_DIR"), "/chapters.rs"));

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SpanStyle {
    #[default]
    Plain,
    Strong,
    Emphasis,
    Code,
    Link,
}

#[derive(Clone, Debug, Default)]
pub struct Span {
    pub text: String,
    pub style: SpanStyle,
    pub target: Option<String>,
}

#[derive(Clone, Debug, Default)]
pub struct ListItem {
    pub spans: Vec<Span>,
    pub depth: usize,
    pub done: Option<bool>,
}

#[derive(Clone, Debug)]
pub enum Block {
    Heading {
        level: u8,
        spans: Vec<Span>,
        anchor: String,
    },
    Paragraph {
        spans: Vec<Span>,
    },
    List {
        ordered: bool,
        start: u64,
        items: Vec<ListItem>,
    },
    Code {
        language: String,
        lines: Vec<String>,
    },
    Quote {
        spans: Vec<Span>,
    },
    Table {
        headers: Vec<Vec<Span>>,
        rows: Vec<Vec<Vec<Span>>>,
    },
    Image {
        path: String,
        caption: String,
    },
    Rule,
    Stage {
        demo: String,
        title: String,
        caption: String,
    },
}

#[derive(Clone, Debug)]
pub struct Outline {
    pub level: u8,
    pub title: String,
    pub anchor: String,
    pub demo: Option<String>,
}

#[derive(Clone, Debug)]
pub struct Chapter {
    pub part: Option<&'static str>,
    pub title: &'static str,
    pub depth: usize,
    pub path: &'static str,
    pub number: String,
    pub blocks: Vec<Block>,
    pub outline: Vec<Outline>,
    pub demo: Option<String>,
    pub footnotes: Vec<(String, Vec<Span>)>,
}

pub fn anchor_of(spans: &[Span]) -> String {
    let text: String = spans.iter().map(|span| span.text.as_str()).collect();
    let mut anchor = String::with_capacity(text.len());
    let mut dash = false;
    for character in text.chars() {
        if character.is_alphanumeric() {
            for lower in character.to_lowercase() {
                anchor.push(lower);
            }
            dash = false;
        } else if !dash && !anchor.is_empty() {
            anchor.push('-');
            dash = true;
        }
    }
    while anchor.ends_with('-') {
        anchor.pop();
    }
    anchor
}

pub struct ImageSource {
    pub name: &'static str,
    pub bytes: &'static [u8],
}

include!(concat!(env!("OUT_DIR"), "/images.rs"));
