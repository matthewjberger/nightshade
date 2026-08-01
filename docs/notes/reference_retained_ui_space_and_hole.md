---
name: reference_retained_ui_space_and_hole
description: A lone space measures as zero width in the retained UI, and a world rectangle must be left unpainted by every layer above it
type: reference
---

Two things about the retained UI that are easy to lose an afternoon to.

**A lone space measures as nothing.** `measure_text_width_with_kind` reads
`line_w` off the shaped buffer, and the shaper drops whitespace at the end of a
line, so measuring `" "` returns 0. Take the difference instead:
`measure("l l") - measure("ll")`. This matters wherever prose is laid out one
word at a time, because a span holding only a space then collapses and two
words run together.

**A hole is a hole in every layer.** The world reaches the reader through a
node that paints nothing, so nothing above OR BEHIND that node in the same
subtree may paint either. A card drawn around a stage must leave its own
background transparent and paint only its header and its control strip.
Otherwise the card's fill shows through the transparent stage and the scene
never appears. [[project_book_app]] does this.

Related: [[reference_retained_ui_enable]]
