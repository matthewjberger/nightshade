---
name: reference_index_in_a_uv_needs_rounding
description: An integer carried in a texture coordinate is interpolated, so floor on it flips at whole numbers. Round it and sample the middle of the cell
type: reference
---

A whole number packed into a vertex texture coordinate (an atlas cell index,
a layer, a table row) is still interpolated across the triangle, even when all
three corners carry the same value. Perspective-correct interpolation is exact
only in theory, and in practice the fragment gets `416.0` plus or minus an epsilon.

`floor(index / columns)` then sits exactly on a boundary and flips a whole row
on the wrong side of it. The symptom is grit: scattered fragments drawing a
texel from some other cell, worse at distance and in motion, easy to mistake
for shadow acne.

Round first and sample the middle of the cell:

```wgsl
let whole = floor(index + 0.5);
let row = floor((whole + 0.5) / columns);
let column = whole - row * columns;
```

`@interpolate(flat)` is the other cure, but only when nothing else needs that
attribute interpolated.
