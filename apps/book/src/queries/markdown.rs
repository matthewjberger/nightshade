use pulldown_cmark::{CodeBlockKind, Event, HeadingLevel, Options, Parser, Tag, TagEnd};

use crate::data::book::{Block, ListItem, Span, SpanStyle, anchor_of};

pub struct Parsed {
    pub blocks: Vec<Block>,
    pub footnotes: Vec<(String, Vec<Span>)>,
}

pub fn parse_blocks(markdown: &str) -> Parsed {
    let mut state = ParseState::default();
    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_FOOTNOTES);
    options.insert(Options::ENABLE_TASKLISTS);

    for event in Parser::new_ext(markdown, options) {
        read_event(&mut state, event);
    }

    Parsed {
        blocks: state.blocks,
        footnotes: state.footnotes,
    }
}

fn read_event(state: &mut ParseState, event: Event) {
    match event {
        Event::Start(Tag::Heading { level, .. }) => {
            state.spans.clear();
            state.mode = Mode::Heading(heading_level(level));
        }
        Event::End(TagEnd::Heading(_)) => {
            let spans = state.take_spans();
            if !spans.is_empty() {
                let level = match state.mode {
                    Mode::Heading(level) => level,
                    _ => 1,
                };
                let anchor = anchor_of(&spans);
                state.blocks.push(Block::Heading {
                    level,
                    spans,
                    anchor,
                });
            }
            state.mode = Mode::None;
        }
        Event::Start(Tag::Paragraph) => {
            if state.lists.is_empty() && state.footnote.is_none() {
                state.spans.clear();
                state.mode = Mode::Paragraph;
            }
        }
        Event::End(TagEnd::Paragraph) => {
            if !state.lists.is_empty() || state.footnote.is_some() {
                return;
            }
            let spans = state.take_spans();
            if !spans.is_empty() {
                let block = if state.quote_depth > 0 {
                    Block::Quote { spans }
                } else {
                    Block::Paragraph { spans }
                };
                state.blocks.push(block);
            }
            state.mode = Mode::None;
        }
        Event::Start(Tag::BlockQuote(_)) => state.quote_depth += 1,
        Event::End(TagEnd::BlockQuote(_)) => {
            state.quote_depth = state.quote_depth.saturating_sub(1)
        }
        Event::Start(Tag::CodeBlock(kind)) => {
            state.code.clear();
            state.language = match kind {
                CodeBlockKind::Fenced(info) => {
                    info.split_whitespace().next().unwrap_or("").to_string()
                }
                CodeBlockKind::Indented => String::new(),
            };
            state.mode = Mode::Code;
        }
        Event::End(TagEnd::CodeBlock) => {
            let body = std::mem::take(&mut state.code);
            let block = if state.language == "demo" {
                read_stage(&body)
            } else {
                Block::Code {
                    language: state.language.clone(),
                    lines: body.lines().map(|line| line.to_string()).collect(),
                }
            };
            state.blocks.push(block);
            state.mode = Mode::None;
        }
        Event::Start(Tag::List(start)) => state.lists.push(ListBuild {
            ordered: start.is_some(),
            start: start.unwrap_or(1),
            items: Vec::new(),
        }),
        Event::End(TagEnd::List(_)) => {
            let Some(list) = state.lists.pop() else {
                return;
            };
            if list.items.is_empty() {
                return;
            }
            match state.lists.last_mut() {
                Some(parent) => parent.items.extend(list.items),
                None => state.blocks.push(Block::List {
                    ordered: list.ordered,
                    start: list.start,
                    items: list.items,
                }),
            }
        }
        Event::Start(Tag::Item) => {
            state.spans.clear();
            state.mode = Mode::Item;
            state.item_done = None;
        }
        Event::End(TagEnd::Item) => {
            let spans = state.take_spans();
            let depth = state.lists.len().saturating_sub(1);
            let done = state.item_done.take();
            if let Some(list) = state.lists.last_mut()
                && !spans.is_empty()
            {
                list.items.push(ListItem { spans, depth, done });
            }
            state.mode = Mode::None;
        }
        Event::TaskListMarker(done) => state.item_done = Some(done),
        Event::Start(Tag::Table(_)) => {
            state.table = Some(TableBuild::default());
            state.mode = Mode::Table;
        }
        Event::End(TagEnd::Table) => {
            if let Some(table) = state.table.take()
                && !table.headers.is_empty()
            {
                state.blocks.push(Block::Table {
                    headers: table.headers,
                    rows: table.rows,
                });
            }
            state.mode = Mode::None;
        }
        Event::Start(Tag::TableHead) => state.in_header = true,
        Event::End(TagEnd::TableHead) => state.in_header = false,
        Event::Start(Tag::TableRow) => state.row.clear(),
        Event::End(TagEnd::TableRow) => {
            let row = std::mem::take(&mut state.row);
            if let Some(table) = state.table.as_mut() {
                table.rows.push(row);
            }
        }
        Event::Start(Tag::TableCell) => state.spans.clear(),
        Event::End(TagEnd::TableCell) => {
            let spans = state.take_spans();
            if state.in_header {
                if let Some(table) = state.table.as_mut() {
                    table.headers.push(spans);
                }
            } else {
                state.row.push(spans);
            }
        }
        Event::Start(Tag::Image { dest_url, .. }) => {
            state.image = Some(dest_url.to_string());
            state.spans.clear();
        }
        Event::End(TagEnd::Image) => {
            let Some(path) = state.image.take() else {
                return;
            };
            let caption: String = state
                .take_spans()
                .iter()
                .map(|span| span.text.clone())
                .collect();
            state.blocks.push(Block::Image { path, caption });
        }
        Event::Start(Tag::FootnoteDefinition(name)) => {
            state.footnote = Some(name.to_string());
            state.spans.clear();
        }
        Event::End(TagEnd::FootnoteDefinition) => {
            let Some(name) = state.footnote.take() else {
                return;
            };
            let spans = state.take_spans();
            state.footnotes.push((name, spans));
        }
        Event::FootnoteReference(name) => {
            let index = state.footnotes.len() + state.pending_references + 1;
            state.pending_references += 1;
            state.push_span(&format!("[{index}]"), SpanStyle::Emphasis, None);
            let _ = name;
        }
        Event::Start(Tag::Strong) => state.strong += 1,
        Event::End(TagEnd::Strong) => state.strong = state.strong.saturating_sub(1),
        Event::Start(Tag::Emphasis) => state.emphasis += 1,
        Event::End(TagEnd::Emphasis) => state.emphasis = state.emphasis.saturating_sub(1),
        Event::Start(Tag::Link { dest_url, .. }) => {
            state.link += 1;
            state.link_target = Some(dest_url.to_string());
        }
        Event::End(TagEnd::Link) => {
            state.link = state.link.saturating_sub(1);
            if state.link == 0 {
                state.link_target = None;
            }
        }
        Event::Text(text) => {
            if matches!(state.mode, Mode::Code) {
                state.code.push_str(&text);
            } else {
                let style = state.inline_style();
                let target = state.link_target.clone();
                state.push_span(&text, style, target);
            }
        }
        Event::Code(text) => state.push_span(&text, SpanStyle::Code, None),
        Event::SoftBreak | Event::HardBreak => {
            if matches!(state.mode, Mode::Code) {
                state.code.push('\n');
            } else {
                let style = state.inline_style();
                state.push_span(" ", style, None);
            }
        }
        Event::Rule => state.blocks.push(Block::Rule),
        _ => {}
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum Mode {
    #[default]
    None,
    Heading(u8),
    Paragraph,
    Code,
    Item,
    Table,
}

struct ListBuild {
    ordered: bool,
    start: u64,
    items: Vec<ListItem>,
}

#[derive(Default)]
struct TableBuild {
    headers: Vec<Vec<Span>>,
    rows: Vec<Vec<Vec<Span>>>,
}

#[derive(Default)]
struct ParseState {
    blocks: Vec<Block>,
    spans: Vec<Span>,
    lists: Vec<ListBuild>,
    table: Option<TableBuild>,
    row: Vec<Vec<Span>>,
    in_header: bool,
    image: Option<String>,
    footnote: Option<String>,
    footnotes: Vec<(String, Vec<Span>)>,
    pending_references: usize,
    item_done: Option<bool>,
    code: String,
    language: String,
    link_target: Option<String>,
    mode: Mode,
    quote_depth: usize,
    strong: usize,
    emphasis: usize,
    link: usize,
}

impl ParseState {
    fn inline_style(&self) -> SpanStyle {
        if self.strong > 0 {
            SpanStyle::Strong
        } else if self.emphasis > 0 {
            SpanStyle::Emphasis
        } else if self.link > 0 {
            SpanStyle::Link
        } else {
            SpanStyle::Plain
        }
    }

    fn push_span(&mut self, text: &str, style: SpanStyle, target: Option<String>) {
        if text.is_empty() {
            return;
        }
        if let Some(last) = self.spans.last_mut()
            && last.style == style
            && last.target == target
        {
            last.text.push_str(text);
            return;
        }
        self.spans.push(Span {
            text: text.to_string(),
            style,
            target,
        });
    }

    fn take_spans(&mut self) -> Vec<Span> {
        let mut spans = std::mem::take(&mut self.spans);
        for span in &mut spans {
            span.text = span.text.replace('\n', " ");
        }
        spans.retain(|span| !span.text.trim().is_empty() || span.style == SpanStyle::Plain);
        carry_spaces_back(&mut spans);
        if let Some(first) = spans.first_mut() {
            first.text = first.text.trim_start().to_string();
        }
        if let Some(last) = spans.last_mut() {
            last.text = last.text.trim_end().to_string();
        }
        spans.retain(|span| !span.text.is_empty());
        spans
    }
}

fn carry_spaces_back(spans: &mut Vec<Span>) {
    let mut carried: Vec<Span> = Vec::with_capacity(spans.len());
    for mut span in std::mem::take(spans) {
        let spaces = span.text.len() - span.text.trim_start_matches(' ').len();
        if spaces == 0 || carried.is_empty() {
            carried.push(span);
            continue;
        }

        let moved = span.text[..spaces].to_string();
        span.text = span.text[spaces..].to_string();

        match carried.last_mut() {
            Some(previous) if previous.style == SpanStyle::Plain => previous.text.push_str(&moved),
            _ => carried.push(Span {
                text: moved,
                style: SpanStyle::Plain,
                target: None,
            }),
        }
        carried.push(span);
    }
    *spans = carried;
}

fn heading_level(level: HeadingLevel) -> u8 {
    match level {
        HeadingLevel::H1 => 1,
        HeadingLevel::H2 => 2,
        HeadingLevel::H3 => 3,
        HeadingLevel::H4 => 4,
        HeadingLevel::H5 => 5,
        HeadingLevel::H6 => 6,
    }
}

fn read_stage(body: &str) -> Block {
    let mut demo = String::new();
    let mut title = String::new();
    let mut caption = String::new();
    for line in body.lines() {
        let Some((key, value)) = line.split_once(':') else {
            continue;
        };
        let value = value.trim().to_string();
        match key.trim() {
            "id" => demo = value,
            "title" => title = value,
            "caption" => caption = value,
            _ => {}
        }
    }
    Block::Stage {
        demo,
        title,
        caption,
    }
}
