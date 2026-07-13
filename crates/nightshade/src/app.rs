//! The plugin-composed application builder.
//!
//! An [`App`] assembles a program from [`Plugin`]s: the engine's
//! capabilities ship as plugins, and the game itself is one more plugin
//! added in the builder. An empty app runs its startup schedule and one
//! pass of the stages, then exits; [`WindowPlugin`] adds the event loop
//! and a window;
//! [`RenderPlugin`] adds the renderer; omitting [`WindowPlugin`] while
//! keeping [`RenderPlugin`] renders against a hidden window. Behavior
//! plugins toggle the engine capabilities their systems need, and
//! [`DefaultPlugins`] bundles the standard set.
//!
//! ```ignore
//! use nightshade::prelude::*;
//!
//! struct MyGamePlugin;
//!
//! impl Plugin for MyGamePlugin {
//!     fn build(&self, app: &mut App) {
//!         app.world.ecs.add_world_at(GAME, register_game_components());
//!         app.insert_resource(MyGame::default());
//!         app.add_startup_system("initialize", |world| {
//!             game_scope::<MyGame, _>(world, systems::initialize);
//!         });
//!         app.add_system("update", "tick", |world| {
//!             game_scope::<MyGame, _>(world, systems::tick);
//!         });
//!     }
//! }
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     App::new()
//!         .add_plugins(DefaultPlugins)
//!         .add_plugin(MyGamePlugin)
//!         .run()
//! }
//! ```

use crate::ecs::world::World;
use crate::state::{RenderResources, State};

/// A unit of app composition. Engine capabilities and games alike
/// implement this and register their worlds, resources, systems, and
/// hooks against the [`App`] in `build`.
pub trait Plugin {
    fn build(&self, app: &mut App);
}

/// A set of plugins added together, in a fixed order.
pub trait PluginGroup {
    fn build(&self, app: &mut App);
}

type RenderGraphConfigFn = Box<
    dyn FnMut(
        &mut crate::render::wgpu::rendergraph::RenderGraph<
            crate::render::wgpu::render_configs::RenderInputs,
        >,
        &wgpu::Device,
        wgpu::TextureFormat,
        RenderResources,
    ),
>;
type PreRenderFn = Box<dyn FnMut(&mut crate::render::wgpu::WgpuRenderer, &mut World)>;
type UpdateRenderGraphFn = Box<
    dyn FnMut(
        &mut crate::render::wgpu::rendergraph::RenderGraph<
            crate::render::wgpu::render_configs::RenderInputs,
        >,
        &World,
    ),
>;

/// The plugin-composed application: the world, a startup schedule that
/// runs once, and named per-frame stages that plugins push systems into.
///
/// Stage order is declared here once: `"first"`, `"update"`, and
/// `"late_update"`, all running in that order each frame BEFORE the
/// engine's frame schedule (physics, animation, transforms, retained
/// UI) and rendering, exactly where game logic ran historically. Game
/// logic belongs in `"update"`; `"late_update"` is for systems that
/// must observe every update system's writes; work that must follow
/// the engine systems hooks into the frame schedule itself with
/// `schedule_insert_before`/`schedule_insert_after`.
pub struct App {
    pub world: World,
    pub startup: freecs::Schedule<World>,
    pub stages: freecs::Stages<World>,
    windowed: bool,
    rendered: bool,
    render_graph_config: Vec<RenderGraphConfigFn>,
    pre_render: Vec<PreRenderFn>,
    update_render_graph: Vec<UpdateRenderGraphFn>,
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl App {
    pub fn new() -> Self {
        let mut stages = freecs::Stages::new();
        stages
            .add_stage("first")
            .add_stage("update")
            .add_stage("late_update");
        Self {
            world: World::default(),
            startup: freecs::Schedule::new(),
            stages,
            windowed: false,
            rendered: false,
            render_graph_config: Vec::new(),
            pre_render: Vec::new(),
            update_render_graph: Vec::new(),
        }
    }

    pub fn add_plugin(mut self, plugin: impl Plugin) -> Self {
        plugin.build(&mut self);
        self
    }

    pub fn add_plugins(mut self, group: impl PluginGroup) -> Self {
        group.build(&mut self);
        self
    }

    /// Pushes a system onto a declared stage. Panics on an undeclared
    /// stage name, listing the declared set.
    pub fn add_system(
        &mut self,
        stage: &str,
        name: &'static str,
        system: impl FnMut(&mut World) + Send + 'static,
    ) -> &mut Self {
        self.stages.stage_mut(stage).push(name, system);
        self
    }

    /// Pushes a system onto the startup schedule, which runs once after
    /// the window and renderer come up (or immediately for an app with
    /// no runner).
    pub fn add_startup_system(
        &mut self,
        name: &'static str,
        system: impl FnMut(&mut World) + Send + 'static,
    ) -> &mut Self {
        self.startup.push(name, system);
        self
    }

    /// Inserts a group-level resource, readable from any system through
    /// [`game_scope`] or `world.ecs.resource`/`resource_mut`.
    pub fn insert_resource<T: Send + Sync + 'static>(&mut self, value: T) -> &mut Self {
        self.world.ecs.insert_resource(value);
        self
    }

    /// Adds a render graph configuration hook, called once when the
    /// renderer is created. Hooks from every plugin run in the order they
    /// were added.
    pub fn add_render_graph_config(
        &mut self,
        hook: impl FnMut(
            &mut crate::render::wgpu::rendergraph::RenderGraph<
                crate::render::wgpu::render_configs::RenderInputs,
            >,
            &wgpu::Device,
            wgpu::TextureFormat,
            RenderResources,
        ) + 'static,
    ) -> &mut Self {
        self.render_graph_config.push(Box::new(hook));
        self
    }

    /// Adds a pre-render hook, called each frame before the render graph
    /// executes. Hooks from every plugin run in the order they were added.
    pub fn on_pre_render(
        &mut self,
        hook: impl FnMut(&mut crate::render::wgpu::WgpuRenderer, &mut World) + 'static,
    ) -> &mut Self {
        self.pre_render.push(Box::new(hook));
        self
    }

    /// Adds a per-frame render graph update hook. Hooks from every plugin
    /// run in the order they were added.
    pub fn on_update_render_graph(
        &mut self,
        hook: impl FnMut(
            &mut crate::render::wgpu::rendergraph::RenderGraph<
                crate::render::wgpu::render_configs::RenderInputs,
            >,
            &World,
        ) + 'static,
    ) -> &mut Self {
        self.update_render_graph.push(Box::new(hook));
        self
    }

    pub(crate) fn set_windowed(&mut self) {
        self.windowed = true;
    }

    pub(crate) fn set_rendered(&mut self) {
        self.rendered = true;
    }

    /// Runs the app. With neither [`WindowPlugin`] nor [`RenderPlugin`],
    /// the startup schedule and one pass of the stages run against the
    /// world and the program returns: the do-nothing composition. With
    /// either, the winit event
    /// loop drives the startup schedule once and then the stages every
    /// frame; without [`WindowPlugin`] the window stays hidden, and
    /// without [`RenderPlugin`] no renderer is created.
    pub fn run(mut self) -> Result<(), Box<dyn std::error::Error>> {
        if !self.windowed && !self.rendered {
            let mut world = self.world;
            self.startup.run(&mut world);
            self.stages.run(&mut world);
            return Ok(());
        }

        self.world.resources.window.start_hidden = !self.windowed;
        self.world.resources.window.create_renderer = self.rendered;

        let (world, state) = self.into_parts();
        crate::run::launch_with_world(state, world)
    }

    /// Splits the composed app into its world and a [`State`] that drives
    /// the startup schedule at initialize and the stages every frame, for
    /// custom runners like offscreen workers that own their own loop.
    pub fn into_parts(self) -> (World, impl State + 'static) {
        let App {
            world,
            startup,
            stages,
            render_graph_config,
            pre_render,
            update_render_graph,
            ..
        } = self;

        (
            world,
            StageState {
                startup,
                stages,
                render_graph_config,
                pre_render,
                update_render_graph,
            },
        )
    }
}

/// Temporarily removes a group resource so a system can hold it alongside
/// full `&mut World` access, then puts it back. The safe equivalent of a
/// resource scope: systems keep the plain `(game, world)` signature with
/// no aliasing and no `unsafe`. The resource is reinserted even if `body`
/// unwinds.
///
/// Panics if the resource was never inserted or is already scoped.
pub fn game_scope<T: Send + Sync + 'static, R>(
    world: &mut World,
    body: impl FnOnce(&mut T, &mut World) -> R,
) -> R {
    let value = world.ecs.remove_resource::<T>().unwrap_or_else(|| {
        panic!(
            "game_scope: resource {} is not inserted",
            std::any::type_name::<T>()
        )
    });
    let mut scope = ScopedResource {
        world,
        value: Some(value),
    };
    body(
        scope
            .value
            .as_mut()
            .expect("game_scope: scoped resource missing"),
        scope.world,
    )
}

struct ScopedResource<'world, T: Send + Sync + 'static> {
    world: &'world mut World,
    value: Option<T>,
}

impl<T: Send + Sync + 'static> Drop for ScopedResource<'_, T> {
    fn drop(&mut self) {
        if let Some(value) = self.value.take() {
            self.world.ecs.insert_resource(value);
        }
    }
}

/// The [`State`] adapter that drives an [`App`]'s schedules through the
/// engine runner: startup once at initialize, stages every frame, render
/// hooks forwarded.
struct StageState {
    startup: freecs::Schedule<World>,
    stages: freecs::Stages<World>,
    render_graph_config: Vec<RenderGraphConfigFn>,
    pre_render: Vec<PreRenderFn>,
    update_render_graph: Vec<UpdateRenderGraphFn>,
}

impl State for StageState {
    fn initialize(&mut self, world: &mut World) {
        self.startup.run(world);
    }

    fn run_systems(&mut self, world: &mut World) {
        self.stages.run(world);
    }

    fn configure_render_graph(
        &mut self,
        graph: &mut crate::render::wgpu::rendergraph::RenderGraph<
            crate::render::wgpu::render_configs::RenderInputs,
        >,
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        resources: RenderResources,
    ) {
        for hook in &mut self.render_graph_config {
            hook(graph, device, format, resources);
        }
    }

    fn pre_render(&mut self, renderer: &mut crate::render::wgpu::WgpuRenderer, world: &mut World) {
        for hook in &mut self.pre_render {
            hook(renderer, world);
        }
    }

    fn update_render_graph(
        &mut self,
        graph: &mut crate::render::wgpu::rendergraph::RenderGraph<
            crate::render::wgpu::render_configs::RenderInputs,
        >,
        world: &World,
    ) {
        for hook in &mut self.update_render_graph {
            hook(graph, world);
        }
    }
}

/// Adds the winit event loop and a visible window.
pub struct WindowPlugin {
    pub title: String,
    pub size: Option<(u32, u32)>,
}

impl Default for WindowPlugin {
    fn default() -> Self {
        Self {
            title: "Nightshade".to_string(),
            size: None,
        }
    }
}

impl Plugin for WindowPlugin {
    fn build(&self, app: &mut App) {
        app.world.resources.window.title = self.title.clone();
        if self.size.is_some() {
            app.world.resources.window.initial_size = self.size;
        }
        app.set_windowed();
    }
}

/// Adds the renderer. With [`WindowPlugin`] it renders to the window;
/// without, it renders against a hidden window for headless capture.
pub struct RenderPlugin;

impl Plugin for RenderPlugin {
    fn build(&self, app: &mut App) {
        app.set_rendered();
    }
}

/// Enables the immediate-draw and retained UI systems.
pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.world.resources.user_interface.enabled = true;
        app.world.resources.retained_ui.enabled = true;
    }
}

/// Enables the physics simulation.
#[cfg(feature = "physics")]
pub struct PhysicsPlugin;

#[cfg(feature = "physics")]
impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.world.resources.physics.enabled = true;
    }
}

/// The standard composition: a visible window, the renderer, and the UI
/// systems. Physics stays opt-in through [`PhysicsPlugin`] because most
/// apps drive it per screen.
pub struct DefaultPlugins;

impl PluginGroup for DefaultPlugins {
    fn build(&self, app: &mut App) {
        WindowPlugin::default().build(app);
        RenderPlugin.build(app);
        UiPlugin.build(app);
    }
}
