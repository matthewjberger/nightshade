# nightshade-wgpu

The wgpu backend. It reads the frame description that `nightshade-render` holds and draws it over a render graph, owning every pass and every shader itself. The crate shows two paths and nothing else, `nightshade_wgpu::wgpu` for the GPU API and `nightshade_wgpu::backend` for the graph, the passes and the install points, so the pass tree is not public API.

## Composing it

```rust
let mut app = App::new();
app.add_plugins(nightshade_wgpu::DefaultPlugins::<nightshade_cli::NoArguments>::new());
app.add_plugin(CameraControllerPlugin)
    .add_plugin(ManyLightsPlugin)
    .run()
```

`DefaultPlugins` is the window group with this backend's `RenderPlugin` set as its renderer. `RenderPlugin` composes `GraphicsPlugin`, inserts the frames the capability crates publish into, `DecalFrame`, `ParticleFrame`, `SpriteFrame`, `PortalFrame`, `SdfFrame`, `PaintFrame` and the acoustics resources, installs its own passes for each of them, and sets the windowed runner. `vsync` and `frame_rate_limit` on the plugin write `FramePacing`.

## Using it

```rust
use nightshade_wgpu::RenderAppExt;

impl Plugin for BoidsWgpuPlugin {
    fn build(&self, app: &mut App) {
        app.add_render_graph_config(install_boids_pass);
        app.on_pre_render(hand_over_flock);
    }
}

fn install_boids_pass(
    graph: &mut nightshade_wgpu::backend::rendergraph::RenderGraph<
        nightshade_wgpu::backend::render_configs::RenderInputs,
    >,
    build: nightshade_wgpu::backend::pass_build::PassBuild,
    _color_format: nightshade_wgpu::wgpu::TextureFormat,
) {
    let Some(color) = render_graph_resource_id(graph, "scene_color") else {
        return;
    };
    let pass = BoidsGpuPass::new(build, MAX_BOID_COUNT as u32);
    install_scene_pass(graph, Box::new(pass), &[("color", color)]);
}
```

`RenderAppExt` carries the four extension points, `add_render_graph_config`, `on_update_render_graph`, `on_pre_render` and `on_window_event`. A graph config hook receives the graph, the builder and the surface format, asks the graph for the targets it reads by name with `render_graph_resource_id`, and places its pass with `install_scene_pass`, `install_lit_pass` or `add_overlay_sub_pass`. A pre-render hook is a system whose first parameter is `nightshade_wgpu::pre_render::Renderer`, followed by the resources and the one query it reads, and it reaches its pass again with `render_graph_get_pass_mut` and a downcast. `into_parts` takes an `App` back apart into the world, the stage state and the render hooks, which is how a host drives the composed program from a loop of its own.

## Taking it from another crate

The crate does not split at a `contract` feature, because `nightshade-render` is its contract. A capability crate publishes its frame into `nightshade-render` or its own contract and never names this crate. The three crates that do name it take it whole for the two public paths.
