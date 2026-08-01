---
name: reference_viewer_layout_rebuilds_on_width
description: The viewer tears its interface down and builds it again when the window crosses 700 logical pixels, because a boundary is fixed when the tree is built and no command moves one afterward
type: reference
---

`apps/viewer` reads `layout_of(layout_width(&window))` in `build_ui` at
startup and again in `relayout` on every frame. Every width the interface
places by hand comes from that value. Below 700 logical pixels the toolbar is
two rows, a panel takes the width it is given, the panels start closed, and
opening one closes the other.

A panel's boundary is fixed when the tree is built, and the retained UI has no
command that moves a boundary afterward, so a window that crosses the width
gets a new tree rather than a moved one: `relayout` despawns the interface
root and the show button, builds the tree again, drops every handle, and
marks every list dirty so the next frame fills it again. The search text and
the tree filter go with the old tree. Thumbnails and texture previews stay in
the atlas, because their slots live on the resources rather than on the tree.

A phone lays out in about 475 logical pixels: 1248 physical at density 420 is
a scale factor of 2.625.

Related: [[reference_android_manifest_needs_internet]]
