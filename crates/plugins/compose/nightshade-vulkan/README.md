# nightshade-vulkan

The Vulkan backend. It reads the same frame description out of `nightshade-render` that the other backend reads, and draws it over an ordered pass list on Vulkan 1.3 dynamic rendering. It exists for the task and mesh shaders and the ray traced work WGSL cannot express. The crate shows two paths, `nightshade_vulkan::ash` for the GPU API and `nightshade_vulkan::backend` for the renderer, the pass types and the install points.

## Composing it

```rust
App::new()
    .add_plugin(nightshade_vulkan::RenderPlugin::default())
    .add_plugin(nightshade_window::WindowPlugin::default())
    .add_plugin(nightshade_cli::CliPlugin::<Arguments>::default())
    .add_plugin(nightshade_log::LogPlugin::default())
    .add_plugin(nightshade_camera::CameraPlugin)
    .add_plugin(TerrainPlugin)
    .run()
```

`RenderPlugin` composes `GraphicsPlugin`, inserts `FramePacing`, `TextState`, `PaintFrame`, `SpriteFrame` and its own `TextMeshCache`, `GpuTimings` and `GpuDrawn`, and installs its eleven passes, sky, scene, grid, lines, sprites, particles, text, decals, post, outline and paint, each with the pre-render hooks that fill them. `nightshade_vulkan::DefaultPlugins` is the same five plugin group with this backend set as its renderer, for a program that wants the group rather than the five lines.

## Using it

```rust
use nightshade_vulkan::VulkanAppExt;
use nightshade_vulkan::backend::{VulkanRenderer, install_scene_pass, pass_setup, pass_with_context};

impl Plugin for BoidsVulkanPlugin {
    fn build(&self, app: &mut App) {
        app.install_vulkan_pass(install_boids_pass);
        app.on_vulkan_pre_render(hand_over_flock);
    }
}

fn install_boids_pass(renderer: &mut VulkanRenderer) {
    let setup = pass_setup(renderer);
    match pass::create_pass(&setup, MAX_BOID_COUNT as u32) {
        Ok(pass) => install_scene_pass(renderer, Box::new(pass)),
        Err(error) => tracing::error!("the boids pass was not installed: {error}"),
    }
}
```

`VulkanAppExt` carries three extension points, `install_vulkan_pass`, `on_vulkan_pre_render` and `on_vulkan_window_event`. The hook names differ from the other backend's, so naming both in one program is a compile error rather than a silent clash. An install hook takes the renderer, reads its `PassSetup` with `pass_setup`, builds a `Pass` and hands it to `install_scene_pass` or `install_swapchain_pass`. A pre-render hook is a system whose first parameter is `nightshade_vulkan::pre_render::Renderer`, and it reaches its own pass again with `pass_with_context` or `pass_mut` by name. `Buffer`, `Image`, `create_buffer`, `write_buffer`, `upload_to_buffer`, `create_shader_module` and `read_spirv` are the memory and shader helpers a pass builds on.

## Taking it from another crate

The crate does not split at a `contract` feature, because `nightshade-render` is its contract. A capability crate publishes its frame and never names this crate, and a program that places a pass of its own takes the whole crate for the two public paths.
