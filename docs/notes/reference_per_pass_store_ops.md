---
name: reference_per_pass_store_ops
description: Store ops are worked out per pass and resource, and a custom PostProcessPass must map scene_color
type: reference
---

The render graph works out its store ops per pass and resource, in
`render_graph_compute_store_ops`, and skips a disabled node.
`render_graph_set_pass_enabled` recomputes only that map, because lifetimes and
aliasing are declared rather than derived, so nothing is reallocated.

A handle carries no baked store op. `get_color_attachment` and
`get_depth_attachment` resolve one from the map and store by default for a key
that is missing, which is what a disabled pass that still executes needs.

`entity_id` and `depth` force their store. Without that, GPU picking reads the
reverse-Z clear of 0.0 as no hit while CPU picking still works, because the
selection outline reads `entity_id` through an in-graph reader.

An app that registers its own `PostProcessPass` maps the `scene_color` slot as
well as `hdr`, or `add_pass` fails with `SlotNotMapped`.
