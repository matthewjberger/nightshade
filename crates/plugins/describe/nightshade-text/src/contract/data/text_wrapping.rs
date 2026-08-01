use crate::contract::data::font_engine::FontEngine;
use crate::contract::data::text_data::FontKind;
use cosmic_text::LayoutRun;

fn visual_segment<'a>(run: &LayoutRun<'a>) -> &'a str {
    let Some(first) = run.glyphs.first() else {
        return "";
    };
    let mut start = first.start;
    let mut end = first.end;
    for glyph in run.glyphs {
        start = start.min(glyph.start);
        end = end.max(glyph.end);
    }
    &run.text[start..end]
}

pub fn wrap_text(
    font_engine: &mut FontEngine,
    text: &str,
    font_size: f32,
    available_width: f32,
    font_kind: FontKind,
) -> String {
    if available_width <= 0.0 || text.is_empty() {
        return text.to_string();
    }
    let line_height = font_size * 1.2;
    let buffer = font_engine.shape_buffer(
        text,
        font_size,
        line_height,
        Some(available_width),
        font_kind,
    );
    let mut result = String::with_capacity(text.len() + 16);
    let mut first = true;
    for run in buffer.layout_runs() {
        if !first {
            result.push('\n');
        }
        first = false;
        result.push_str(visual_segment(&run));
    }
    result
}
