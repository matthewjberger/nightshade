---
name: reference_engine_renders_reverse_z
description: Nightshade renders reverse-Z, so a new pass clears depth to zero and compares with GreaterEqual, and getting it backwards makes geometry look translucent
type: reference
---

Every depth-tested pass in `nightshade-wgpu` uses
`CompareFunction::GreaterEqual` or `Greater`, and the depth target clears to
**0.0**. The camera projections are built to match.

**How it fails when you get it wrong:** a new pass written with the usual
`LESS` and a 1.0 clear does not error, does not trip the validation layer, and
does not fail any gate. Far fragments simply win the depth test, so a dense
object renders as if it were translucent and you see through the front of it
to the back. That is the symptom to recognize.

**How to apply:** when adding a pass in any backend, clear depth to `0.0` and
compare with `GREATER_OR_EQUAL`. Do not copy the depth state from an outside
example. Check an existing pass in the same backend instead.
