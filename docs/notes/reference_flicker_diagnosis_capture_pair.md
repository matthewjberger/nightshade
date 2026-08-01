---
name: reference_flicker_diagnosis_capture_pair
description: Difference two captures of a still scene to find what flickers, with the sponza app and ffmpeg
type: reference
---

To pin down frame to frame flicker, capture two frames of the same still scene
and difference them. `apps/features/sponza` takes `--capture <path>
--capture-frame <n>`, so two runs one frame apart give a pair, and
`ffmpeg -i a.png -i b.png -filter_complex blend=all_mode=difference out.png`
shows what moved. A still camera means everything the difference lights up is a
fault rather than motion.

What a difference usually names: temporal antialiasing clipping its history in
raw HDR values, a bloom that reads the jittered scene color rather than the
resolved frame, and air next to a point light that diverges as one over the
distance squared. Foliage and edge shimmer remain after those and are ordinary.
