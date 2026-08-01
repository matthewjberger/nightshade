---
name: reference_a_browser_rejects_what_the_gates_accept
description: Three WGSL rules that naga and just lint-shaders accept and a browser refuses, so the fault only shows on a page load
type: reference
---

`just lint-shaders` compiles with naga, and naga is more permissive than Tint,
which every browser uses. A shader that passes every gate here can still fail
to compile in a page, and a pipeline that names it is then invalid for the
whole session. Three rules bite:

**A derivative needs uniform control flow.** `dpdx`, `dpdy`, `fwidth` and
`textureSample` must be called where every invocation in the quad still runs. A
branch on a fragment input, or anything after a `return` under such a branch, is
not such a place. A branch on a uniform buffer field is. Hoist the derivative
above the branch, or pass it in when the call sits in a helper. For a read that
only needs level zero, `textureSampleLevel` with `0.0` is legal anywhere.

**`smoothstep` needs `low < high`.** Naga mirrors a reversed pair and draws
correctly, and Tint refuses the module when it can fold both arguments to
constants. Write a descending ramp as `1.0 - smoothstep(low, high, x)`, never
as `smoothstep(high, low, x)`. Only a constant-foldable pair is refused at
compile time, so a reversed pair built from a uniform merely goes undefined.

**A composable module rejects a struct member that ends in a digit.** Inside a
naga_oil module, `pad0`, `corner1` and `_pad0` all fail at device init with
"Composable module identifiers must not require substitution", because naga
reserves `padN` for its own padding. Use a word suffix: `padding_position`.
The same path rejects a WGSL reserved keyword as a field name, `target` and
`sample` among them. A struct composes fine while it sits in a top-level entry
shader, so moving one into a shared module is the moment this bites. WGSL and
bytemuck match by field order, so renaming the WGSL side alone is safe.

The first two show only on a page load. The third shows at renderer init, so
one app reaching its first frame is enough: `apps/rendering/reflections` composes all
four mesh shaders.
