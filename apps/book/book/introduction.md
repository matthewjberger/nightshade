# Introduction

Nightshade is a data-oriented game engine written in Rust. It has no scene
graph you inherit from, no base class, and no manager object. A game is a
world of rows, a set of resources, and a list of functions that run over both
in a fixed order every frame.

This book is that engine, running. Every page is drawn by the same retained
interface a game would use, and the figures are live scenes in the same world
as the text. Nothing on a page is a picture.

```demo
id: spinning_triangle
title: A spinning triangle
caption: Three vertices, one material, one camera. The controls drive the same numbers your game would write.
```

## What the engine is made of

Two crates hold the substrate. `nightshade-ecs` is the storage: entities,
components, resources, change ticks, and the scheduler. `nightshade` sits on
top of it and holds four things, `App`, `Plugin`, `Stage`, and the `World` it
re-exports. That is the whole engine.

Everything else is a plugin crate. The window is one. The renderer is one.
Physics, audio, animation, navigation, and the interface you are reading are
all plugin crates, each depending on `nightshade` while `nightshade` depends
on none of them.

| Layer | Crate | Holds |
| --- | --- | --- |
| Storage | `nightshade-ecs` | Rows, resources, the schedule |
| Substrate | `nightshade` | `App`, `Plugin`, `Stage` |
| Capability | `nightshade-*` | One domain each |

A capability the program never names costs nothing and links nothing.

## How to read the pages

Start at [Installation](getting-started/installation.md), then read a page for
the shape of the thing and the crate for the detail.
The pages are short on purpose. Each one names the types you will write and
shows the smallest program that uses them.

- Use the sidebar to move between chapters
- Use the arrows in the footer, or the left and right arrow keys
- Press `/` to search every heading in the book

> Where a page carries a figure, drive it. The controls under a figure write
> into the same components a system of yours would write, and the scene reacts
> the same way.

The code in this book is Rust, and it compiles. Where a snippet leaves
something out, it says so.

## What the engine draws

The pictures that follow were all taken from apps in this repository, each of
which you can run yourself.

![Sponza, lit by one sun through the windows, with the shadows and the fog the
renderer puts under it](images/sponza-atrium.jpg)

The renderer draws through two backends over one description of a frame. The
scene below is the same engine on Vulkan, where the lights are counted in the
hundreds.

![A few hundred lights over a scene, drawn by the Vulkan backend](images/vulkan-lights.jpg)
