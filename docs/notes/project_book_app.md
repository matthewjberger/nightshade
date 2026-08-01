---
name: project_book_app
description: apps/book is the mdbook-style book drawn by the retained UI, with live demos and a capture flag for checking any page
type: project
---

`apps/book` is the Nightshade book: markdown under `apps/book/book/`,
read at build time by `build.rs` and compiled in, drawn by the retained UI. A
page whose markdown holds a ` ```demo ` fence gets a live scene in place of a
picture. One demo is mounted at a time and `commands/scene.rs` gives back
everything it took, so no demo leaks into the next.

Check any page without clicking: `book.exe --capture <path>` writes one
frame and stops, and `--page <n>`, `--scroll <pixels>` and `--dark` say which
part of which page in which theme. Use it rather than guessing at layout.

The demos turn the tonemapper and the auto exposure off, so a stage reads at
the colors the materials name rather than at whatever the exposure settled on.

Related: [[reference_retained_ui_space_and_hole]]
