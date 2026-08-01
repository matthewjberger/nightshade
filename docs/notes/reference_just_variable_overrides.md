---
name: reference_just_variable_overrides
description: In `just`, command-line `VAR=value` overrides must come BEFORE the recipe name, never after
type: reference
---

In the `just` command runner, `VAR=value` overrides of top-level variables must appear BEFORE the recipe name.

- Right: `just project=mygame build-android`
- Wrong: `just build-android project=mygame` just parses `project=mygame` as a second recipe name and errors with "Justfile does not contain recipe `project=mygame`"

Recipe parameters (defined as `recipe param="default":`) are positional, not keyword passed as `just recipe value`, not `just recipe param=value`.

When writing justfile examples in READMEs or docs, put overrides before the recipe name.
