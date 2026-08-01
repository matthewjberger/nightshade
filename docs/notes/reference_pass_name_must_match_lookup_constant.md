---
name: reference_pass_name_must_match_lookup_constant
description: A render pass reports its name from PassNode::name and is found by a separate lookup string, and a drift between them makes the pass draw nothing with no gate to catch it
type: reference
---

A pre-render sync finds its pass with
`render_graph_get_pass_mut(&mut renderer.graph, SOME_PASS)` and the pass reports
its own identity from `PassNode::name`. Those are two separate strings. When
they disagree the lookup returns `None`, the sync returns early, the draw list
is never handed over, and **the pass draws nothing with no error anywhere**.

No gate catches a drift. `cargo check`, clippy, `dag-check`, `shader-check`,
`lint-wasm` and the isolated builds all stay green while the pass draws nothing.
Only running an app shows it, so rename the constant and the `name` body in one
change.

**How to apply:** `name` returns the same constant the lookup uses, never a
repeated literal. When renaming or moving a pass, grep the old string across the
whole workspace rather than editing the module you are in. A change that only
rewires strings passes every static gate this repository has.
