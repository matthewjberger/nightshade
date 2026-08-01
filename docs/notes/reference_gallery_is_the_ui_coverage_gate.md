---
name: reference_gallery_is_the_ui_coverage_gate
description: apps/ui/gallery is the only caller of the retained UI widget constructors, so a dead-code sweep measures the gallery
type: reference
---

Nothing else in the repository calls `UiTreeBuilder::add_*`. A dead code sweep
over the retained UI that asks "does anything call this" is really asking "does
the gallery show this", and a widget the gallery skips reads as unreachable
while its runtime, its component and its widget system are all still there.

Removing such a constructor leaves the runtime orphaned rather than removing a
feature.

The check that works: for every `Ui*Data` component, ask whether anything
outside `components/` builds it with a struct literal or `::default`. The
checks that do not work are `rg -w NAME`, which reads live for any generic
name, and counting occurrences, because the prelude re-exports by glob.

Before removing a widget constructor, ask whether a system still reads what it
was the only way to produce. If one does, remove the runtime with it, or add
gallery coverage and keep both.

Related: [[reference_retained_ui_enable]]
