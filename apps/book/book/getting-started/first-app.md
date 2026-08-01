# Your first app

An empty `App` does nothing. It has no window, no renderer, and no clock,
because the engine holds no capability of its own. You compose what you want.

## The smallest program

```rust
use nightshade::prelude::*;

fn main() {
    App::new()
        .add_plugin(nightshade_log::LogPlugin)
        .add_plugin(nightshade_window::WindowPlugin)
        .add_plugin(nightshade_wgpu::RenderPlugin)
        .run();
}
```

That opens a window and clears it. `WindowPlugin` pulls in the platform, the
scene, and the graphics passes, so the three lines above are the whole
composition.

## Putting something in it

A system is a plain function that names what it touches. Register one on
`Stage::Startup` and it runs once:

```rust
fn spawn_scene(mut commands: Commands, mut resources: Resources) {
    let cache = resources.res_mut::<MeshCache>();
    mesh_cache_insert(cache, "cube".to_string(), create_cube_mesh());

    commands.spawn((
        RenderMesh::new("cube"),
        MaterialRef::default(),
        LocalTransform::default(),
    ));
}
```

Then register it:

```rust
App::new()
    .add_plugin(nightshade_window::WindowPlugin)
    .add_plugin(nightshade_wgpu::RenderPlugin)
    .add_system(Stage::Startup, spawn_scene)
    .run();
```

## Moving it

Game logic goes in `Stage::Update`. A system that turns everything carrying a
`Spin` is one query:

```rust
#[derive(Clone, Copy, Default)]
struct Spin {
    speed: f32,
}

fn turn(time: Res<Time>, mut query: Query<(&Spin, &mut LocalTransform)>) {
    let delta = time.delta_time;
    query.for_each(|_, (spin, transform)| {
        let turn = nalgebra_glm::quat_angle_axis(spin.speed * delta, &Vec3::y());
        transform.rotation = turn * transform.rotation;
    });
}
```

Nothing declared `Spin`. Any `Send + Sync + Default + 'static` type is a
component, and the first write of one appends it to the world's schema. See
[Entities and components](../concepts/entities-and-components.md) for what that
means.

## What runs when

Every frame runs the game stages first, then the engine stages:

1. `Stage::First`
2. `Stage::Update`
3. `Stage::LateUpdate`
4. `Stage::FrameStart`
5. `Stage::FrameUpdate`
6. `Stage::FramePostUpdate`
7. `Stage::Render`
8. `Stage::Last`

Put your work in `Stage::Update` and leave the rest to the plugins that
registered into them.

## Leaving

The escape key is not wired by default. One line adds it:

```rust
.add_system(Stage::FrameUpdate, escape_key_exit_system)
```

That is a system like any other, and it needs no plugin behind it.

## Moving the cube yourself

A transform is three numbers put together: where a thing is, which way it is
turned, and how large it is drawn. Nothing else. The sliders below write
straight into those three, so what you see is the numbers themselves rather
than anything accumulating over time.

```demo
id: transform_parts
title: Moving one cube
caption: One entity, one transform. Each slider writes one part of it, and the frame after draws what the transform now says.
```
