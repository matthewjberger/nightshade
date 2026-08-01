---
name: reference_absolute_scale_three_paths
description: UiLayoutRoot absolute_scale is consumed in three layout paths, so a root that scales in one and not the others leaves stragglers
type: reference
---

`UiLayoutRoot::absolute_scale`, set through `UiTreeBuilder::with_absolute_scale`,
scales a whole UI tree. Three independent paths consume it, and all three must:

1. **Window and boundary `Ab` units**, through `UiEvalContext.absolute_scale` in
   `layout.rs`.
2. **Text glyphs**, through `UiLayoutNode.computed_font_scale`, which layout
   bakes from the root and `emit_text_entry` multiplies into the glyph size,
   along with the path that measures an ellipsis.
3. **Flow and grid children, gaps and padding**, through the taffy path
   (`flow_to_style`, `grid_to_style`, `compute_taffy_children`,
   `build_taffy_subtree`), which takes `absolute_scale`, the root scale times
   the window dpi.

If you scale a root and only some elements follow, find which of the three
paths the stragglers take. At the default scale of 1.0 all three are no-ops,
which is why a missing one hides.
