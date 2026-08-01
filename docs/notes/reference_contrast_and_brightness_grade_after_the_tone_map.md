---
name: reference_contrast_and_brightness_grade_after_the_tone_map
description: Contrast pivots on 0.5 and brightness adds an offset, so both act on the display value after the tone map and the gamma, because on a linear HDR value they push a dark texel negative and the ACES curve turns a negative warm color blue
type: reference
---

`postprocess.wgsl` grades in this order: exposure, saturation, vibrance, the
tone map, the gamma, brightness, contrast, then a clamp to `[0, 1]`.

Saturation and vibrance mix toward luminance, so they are safe on a linear HDR
value. Contrast and brightness need the display value. `apply_contrast` is
`(color - 0.5) * contrast + 0.5`, and `apply_brightness` is `color + offset`.
Each names `0.5` or an offset in display units, so each belongs after the
tone map and the gamma, where `0.5` is mid-grey on the screen.

Applied to a linear HDR value before the tone map, a contrast of `1.12` takes
a dark warm texel such as `(0.03, 0.02, 0.015)` to `(-0.026, -0.038, -0.043)`.
The fitted ACES curve is a rational function that does not clamp its input, and
on a negative input the more negative channel comes out brighter:
`-0.026` maps to `0.007` and `-0.043` maps to `0.027`. The hue inverts, so
every dark warm surface renders blue while the lit surfaces above the pivot
stay warm. An app with a contrast of `1.0` shows nothing of this, so a blue
shadow that appears in one app alone comes from the grading order.
