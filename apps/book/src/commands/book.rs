use crate::data::book::{Block, CHAPTERS, Chapter, Outline};
use crate::queries::markdown::parse_blocks;
use crate::resources::Library;

pub fn load_library(library: &mut Library) {
    let mut counters: Vec<u32> = Vec::new();

    library.chapters = CHAPTERS
        .iter()
        .map(|source| {
            let parsed = parse_blocks(source.markdown);
            Chapter {
                part: source.part,
                title: source.title,
                depth: source.depth,
                path: source.path,
                number: next_number(&mut counters, source.depth),
                outline: read_outline(&parsed.blocks),
                demo: read_leading_demo(&parsed.blocks),
                blocks: parsed.blocks,
                footnotes: parsed.footnotes,
            }
        })
        .collect();
}

fn next_number(counters: &mut Vec<u32>, depth: usize) -> String {
    counters.truncate(depth + 1);
    while counters.len() <= depth {
        counters.push(0);
    }
    counters[depth] += 1;

    let mut number = String::new();
    for (index, value) in counters.iter().enumerate() {
        if index > 0 {
            number.push('.');
        }
        number.push_str(&value.to_string());
    }
    number
}

fn read_outline(blocks: &[Block]) -> Vec<Outline> {
    let mut outline: Vec<Outline> = Vec::new();
    for block in blocks {
        match block {
            Block::Heading {
                level,
                spans,
                anchor,
            } if *level >= 2 && *level <= 3 => outline.push(Outline {
                level: *level,
                title: spans.iter().map(|span| span.text.as_str()).collect(),
                anchor: anchor.clone(),
                demo: None,
            }),
            Block::Stage { demo, .. } => {
                if let Some(heading) = outline.last_mut()
                    && heading.demo.is_none()
                {
                    heading.demo = Some(demo.clone());
                }
            }
            _ => {}
        }
    }
    outline
}

fn read_leading_demo(blocks: &[Block]) -> Option<String> {
    for block in blocks {
        match block {
            Block::Heading { level, .. } if *level >= 2 && *level <= 3 => return None,
            Block::Stage { demo, .. } => return Some(demo.clone()),
            _ => {}
        }
    }
    None
}
