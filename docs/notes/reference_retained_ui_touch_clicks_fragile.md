---
name: reference_retained_ui_touch_clicks_fragile
description: Retained-UI button clicks are unreliable on touch, so hit-test each touch start against the node rect yourself
type: reference
---

The retained-UI click path (`with_interaction` + `ui_button_clicks`) is driven by the mouse state. On touch only the single primary finger is mapped to the mouse (`handle_touch_input` in `nightshade-window/src/platform/input.rs`), and the click needs a press frame then a separate release frame over the same node, so a fast tap whose down+up land in one frame is dropped (picking reads `previous_active` from frame start).

For touch in an app, hit test each fresh touch's `start_position` against the node's `computed_rect` yourself, with `node.computed_rect.contains(point)`, and fire once for each touch id. Keep the mouse `ui_button_clicks` path for the desktop beside it. See also [[reference_absolute_scale_three_paths]].
