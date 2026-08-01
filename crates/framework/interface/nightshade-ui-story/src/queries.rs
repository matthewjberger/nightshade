use crate::resources::StoryStyle;

pub fn height(style: &StoryStyle) -> f32 {
    let rows = style.lines as f32 * style.line_size * 1.35;
    let buttons = (style.links + 1) as f32 * style.row_height;
    let gaps = (style.lines + style.links) as f32 * style.spacing;
    rows + buttons + gaps + style.padding * 2.0
}
