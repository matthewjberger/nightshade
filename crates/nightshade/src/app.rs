//! The plugin-composed application builder.
//!
//! An [`App`] assembles a program from [`Plugin`]s: the engine's
//! capabilities ship as plugins, and the game itself is one more plugin
//! added in the builder. An empty app runs its startup schedule and one
//! pass of the stages, then exits; [`WindowPlugin`] adds the event loop
//! and a window;
//! [`RenderPlugin`] adds the renderer; omitting [`WindowPlugin`] while
//! keeping [`RenderPlugin`] renders against a hidden window. Capability
//! plugins bring their own systems: `AudioPlugin` opens the audio device
//! and installs the audio systems, `GamepadPlugin` polls devices,
//! `PhysicsPlugin` enables the simulation, and `EguiPlugin` the overlay.
//! A capability the program never adds costs nothing at runtime.
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
//!         app.add_system(Stage::Startup, systems::initialize);
//!         app.add_system(Stage::Update, systems::tick);
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
use crate::render::wgpu;
use crate::state::{RenderResources, State};
use nightshade_ecs::dynamic::ResourceHostExt;

/// A unit of app composition. Engine capabilities and games alike
/// implement this and register their worlds, resources, systems, and
/// hooks against the [`App`] in `build`. Each plugin type is added at
/// most once; a second add panics.
pub trait Plugin {
    fn build(&self, app: &mut App);
}

/// A set of plugins added together, in a fixed order. Customize a group
/// at the call site without abandoning it: [`set`](PluginGroup::set)
/// swaps a member's configuration and [`disable`](PluginGroup::disable)
/// drops a member.
///
/// ```ignore
/// App::new()
///     .add_plugins(
///         DefaultPlugins
///             .set(WindowPlugin { title: "My Game".to_string(), size: None })
///             .disable::<LogPlugin>(),
///     )
///     .run()
/// ```
pub trait PluginGroup: Sized {
    fn build(self) -> PluginGroupBuilder;

    fn set<P: Plugin + 'static>(self, plugin: P) -> PluginGroupBuilder {
        self.build().set(plugin)
    }

    fn disable<P: Plugin + 'static>(self) -> PluginGroupBuilder {
        self.build().disable::<P>()
    }
}

/// The ordered members of a [`PluginGroup`], each replaceable by type
/// through [`set`](PluginGroupBuilder::set) and removable through
/// [`disable`](PluginGroupBuilder::disable) before the app adds them.
#[derive(Default)]
pub struct PluginGroupBuilder {
    entries: Vec<PluginGroupEntry>,
}

struct PluginGroupEntry {
    id: std::any::TypeId,
    name: &'static str,
    enabled: bool,
    plugin: Box<dyn Plugin>,
}

impl PluginGroupBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    /// Appends a plugin to the group.
    pub fn add_plugin<P: Plugin + 'static>(mut self, plugin: P) -> Self {
        self.entries.push(PluginGroupEntry {
            id: std::any::TypeId::of::<P>(),
            name: std::any::type_name::<P>(),
            enabled: true,
            plugin: Box::new(plugin),
        });
        self
    }

    /// Replaces the group's existing `P` with the given configuration,
    /// keeping its position. Panics if the group has no `P`.
    pub fn set<P: Plugin + 'static>(mut self, plugin: P) -> Self {
        let entry = self.entry_mut::<P>().unwrap_or_else(|| {
            panic!("{} is not in this plugin group", std::any::type_name::<P>())
        });
        entry.plugin = Box::new(plugin);
        self
    }

    /// Drops `P` from the group. Panics if the group has no `P`.
    pub fn disable<P: Plugin + 'static>(mut self) -> Self {
        let entry = self.entry_mut::<P>().unwrap_or_else(|| {
            panic!("{} is not in this plugin group", std::any::type_name::<P>())
        });
        entry.enabled = false;
        self
    }

    fn entry_mut<P: Plugin + 'static>(&mut self) -> Option<&mut PluginGroupEntry> {
        let id = std::any::TypeId::of::<P>();
        self.entries.iter_mut().find(|entry| entry.id == id)
    }
}

impl PluginGroup for PluginGroupBuilder {
    fn build(self) -> PluginGroupBuilder {
        self
    }
}

macro_rules! impl_plugin_group_for_tuple {
    ($(($plugin_type:ident, $plugin_value:ident)),+) => {
        impl<$($plugin_type: Plugin + 'static),+> PluginGroup for ($($plugin_type,)+) {
            fn build(self) -> PluginGroupBuilder {
                let ($($plugin_value,)+) = self;
                PluginGroupBuilder::new()$(.add_plugin($plugin_value))+
            }
        }
    };
}

impl_plugin_group_for_tuple!((P1, plugin1));
impl_plugin_group_for_tuple!((P1, plugin1), (P2, plugin2));
impl_plugin_group_for_tuple!((P1, plugin1), (P2, plugin2), (P3, plugin3));
impl_plugin_group_for_tuple!((P1, plugin1), (P2, plugin2), (P3, plugin3), (P4, plugin4));
impl_plugin_group_for_tuple!(
    (P1, plugin1),
    (P2, plugin2),
    (P3, plugin3),
    (P4, plugin4),
    (P5, plugin5)
);
impl_plugin_group_for_tuple!(
    (P1, plugin1),
    (P2, plugin2),
    (P3, plugin3),
    (P4, plugin4),
    (P5, plugin5),
    (P6, plugin6)
);
impl_plugin_group_for_tuple!(
    (P1, plugin1),
    (P2, plugin2),
    (P3, plugin3),
    (P4, plugin4),
    (P5, plugin5),
    (P6, plugin6),
    (P7, plugin7)
);
impl_plugin_group_for_tuple!(
    (P1, plugin1),
    (P2, plugin2),
    (P3, plugin3),
    (P4, plugin4),
    (P5, plugin5),
    (P6, plugin6),
    (P7, plugin7),
    (P8, plugin8)
);
impl_plugin_group_for_tuple!(
    (P1, plugin1),
    (P2, plugin2),
    (P3, plugin3),
    (P4, plugin4),
    (P5, plugin5),
    (P6, plugin6),
    (P7, plugin7),
    (P8, plugin8),
    (P9, plugin9)
);
impl_plugin_group_for_tuple!(
    (P1, plugin1),
    (P2, plugin2),
    (P3, plugin3),
    (P4, plugin4),
    (P5, plugin5),
    (P6, plugin6),
    (P7, plugin7),
    (P8, plugin8),
    (P9, plugin9),
    (P10, plugin10)
);
impl_plugin_group_for_tuple!(
    (P1, plugin1),
    (P2, plugin2),
    (P3, plugin3),
    (P4, plugin4),
    (P5, plugin5),
    (P6, plugin6),
    (P7, plugin7),
    (P8, plugin8),
    (P9, plugin9),
    (P10, plugin10),
    (P11, plugin11)
);
impl_plugin_group_for_tuple!(
    (P1, plugin1),
    (P2, plugin2),
    (P3, plugin3),
    (P4, plugin4),
    (P5, plugin5),
    (P6, plugin6),
    (P7, plugin7),
    (P8, plugin8),
    (P9, plugin9),
    (P10, plugin10),
    (P11, plugin11),
    (P12, plugin12)
);
impl_plugin_group_for_tuple!(
    (P1, plugin1),
    (P2, plugin2),
    (P3, plugin3),
    (P4, plugin4),
    (P5, plugin5),
    (P6, plugin6),
    (P7, plugin7),
    (P8, plugin8),
    (P9, plugin9),
    (P10, plugin10),
    (P11, plugin11),
    (P12, plugin12),
    (P13, plugin13)
);
impl_plugin_group_for_tuple!(
    (P1, plugin1),
    (P2, plugin2),
    (P3, plugin3),
    (P4, plugin4),
    (P5, plugin5),
    (P6, plugin6),
    (P7, plugin7),
    (P8, plugin8),
    (P9, plugin9),
    (P10, plugin10),
    (P11, plugin11),
    (P12, plugin12),
    (P13, plugin13),
    (P14, plugin14)
);
impl_plugin_group_for_tuple!(
    (P1, plugin1),
    (P2, plugin2),
    (P3, plugin3),
    (P4, plugin4),
    (P5, plugin5),
    (P6, plugin6),
    (P7, plugin7),
    (P8, plugin8),
    (P9, plugin9),
    (P10, plugin10),
    (P11, plugin11),
    (P12, plugin12),
    (P13, plugin13),
    (P14, plugin14),
    (P15, plugin15)
);
impl_plugin_group_for_tuple!(
    (P1, plugin1),
    (P2, plugin2),
    (P3, plugin3),
    (P4, plugin4),
    (P5, plugin5),
    (P6, plugin6),
    (P7, plugin7),
    (P8, plugin8),
    (P9, plugin9),
    (P10, plugin10),
    (P11, plugin11),
    (P12, plugin12),
    (P13, plugin13),
    (P14, plugin14),
    (P15, plugin15),
    (P16, plugin16)
);

/// The system shapes [`App::add_system`] accepts: a plain `fn(&mut World)`
/// system, a state-and-world `fn(&mut T, &mut World)` game system with
/// [`game_scope`] applied at registration, or a tuple mixing both,
/// registered in tuple order. The `Marker` parameter is inferred from the
/// shape and never written at call sites.
pub trait SystemCollection<Marker> {
    fn push_onto(self, schedule: &mut nightshade_ecs::Schedule<World>);
}

/// Resolves a unique schedule name from a system's type name. Distinct systems
/// that share a type name (the same closure registered per state value, say
/// `on_enter` in a loop) collide otherwise; each gets a `#suffix` so all of
/// them coexist in the schedule instead of the second registration panicking.
fn unique_schedule_name(
    schedule: &nightshade_ecs::Schedule<World>,
    base: &'static str,
) -> &'static str {
    if !schedule.contains(base) {
        return base;
    }
    let mut suffix = 2;
    loop {
        let candidate = format!("{base}#{suffix}");
        if !schedule.contains(&candidate) {
            break Box::leak(candidate.into_boxed_str());
        }
        suffix += 1;
    }
}

pub struct WorldSystemMarker;

pub struct GameSystemMarker<T>(std::marker::PhantomData<T>);

impl<F> SystemCollection<WorldSystemMarker> for F
where
    F: FnMut(&mut World) + Send + 'static,
{
    fn push_onto(self, schedule: &mut nightshade_ecs::Schedule<World>) {
        let name = unique_schedule_name(schedule, std::any::type_name_of_val(&self));
        schedule.push(name, self);
    }
}

impl<T, F> SystemCollection<GameSystemMarker<T>> for F
where
    T: Send + Sync + 'static,
    F: FnMut(&mut T, &mut World) + Send + 'static,
{
    fn push_onto(mut self, schedule: &mut nightshade_ecs::Schedule<World>) {
        let name = unique_schedule_name(schedule, std::any::type_name_of_val(&self));
        schedule.push(name, move |world: &mut World| {
            game_scope(world, |state: &mut T, world| self(state, world));
        });
    }
}

/// Marks a nightshade_ecs system-parameter function, `fn(Res<A>, ResMut<B>, &mut
/// World)`, whose resource arguments resolve out of the ECS resource map
/// while `&mut World` stays free for queries. `World` implements
/// [`nightshade_ecs::dynamic::ResourceHost`], so these register alongside the plain
/// and game system shapes.
pub struct ParamSystemMarker<Marker>(std::marker::PhantomData<Marker>);

impl<Marker, F> SystemCollection<ParamSystemMarker<Marker>> for F
where
    F: nightshade_ecs::system_param::IntoSystem<World, Marker>,
{
    fn push_onto(self, schedule: &mut nightshade_ecs::Schedule<World>) {
        let name = unique_schedule_name(schedule, std::any::type_name_of_val(&self));
        schedule.push(name, self.into_runner());
    }
}

macro_rules! impl_system_collection_for_tuple {
    ($(($marker:ident, $system_type:ident, $system_value:ident)),+) => {
        impl<$($marker),+, $($system_type: SystemCollection<$marker>),+>
            SystemCollection<($($marker,)+)> for ($($system_type,)+)
        {
            fn push_onto(self, schedule: &mut nightshade_ecs::Schedule<World>) {
                let ($($system_value,)+) = self;
                $($system_value.push_onto(schedule);)+
            }
        }
    };
}

impl_system_collection_for_tuple!((M1, S1, system1));
impl_system_collection_for_tuple!((M1, S1, system1), (M2, S2, system2));
impl_system_collection_for_tuple!((M1, S1, system1), (M2, S2, system2), (M3, S3, system3));
impl_system_collection_for_tuple!(
    (M1, S1, system1),
    (M2, S2, system2),
    (M3, S3, system3),
    (M4, S4, system4)
);
impl_system_collection_for_tuple!(
    (M1, S1, system1),
    (M2, S2, system2),
    (M3, S3, system3),
    (M4, S4, system4),
    (M5, S5, system5)
);
impl_system_collection_for_tuple!(
    (M1, S1, system1),
    (M2, S2, system2),
    (M3, S3, system3),
    (M4, S4, system4),
    (M5, S5, system5),
    (M6, S6, system6)
);
impl_system_collection_for_tuple!(
    (M1, S1, system1),
    (M2, S2, system2),
    (M3, S3, system3),
    (M4, S4, system4),
    (M5, S5, system5),
    (M6, S6, system6),
    (M7, S7, system7)
);
impl_system_collection_for_tuple!(
    (M1, S1, system1),
    (M2, S2, system2),
    (M3, S3, system3),
    (M4, S4, system4),
    (M5, S5, system5),
    (M6, S6, system6),
    (M7, S7, system7),
    (M8, S8, system8)
);
impl_system_collection_for_tuple!(
    (M1, S1, system1),
    (M2, S2, system2),
    (M3, S3, system3),
    (M4, S4, system4),
    (M5, S5, system5),
    (M6, S6, system6),
    (M7, S7, system7),
    (M8, S8, system8),
    (M9, S9, system9)
);
impl_system_collection_for_tuple!(
    (M1, S1, system1),
    (M2, S2, system2),
    (M3, S3, system3),
    (M4, S4, system4),
    (M5, S5, system5),
    (M6, S6, system6),
    (M7, S7, system7),
    (M8, S8, system8),
    (M9, S9, system9),
    (M10, S10, system10)
);
impl_system_collection_for_tuple!(
    (M1, S1, system1),
    (M2, S2, system2),
    (M3, S3, system3),
    (M4, S4, system4),
    (M5, S5, system5),
    (M6, S6, system6),
    (M7, S7, system7),
    (M8, S8, system8),
    (M9, S9, system9),
    (M10, S10, system10),
    (M11, S11, system11)
);
impl_system_collection_for_tuple!(
    (M1, S1, system1),
    (M2, S2, system2),
    (M3, S3, system3),
    (M4, S4, system4),
    (M5, S5, system5),
    (M6, S6, system6),
    (M7, S7, system7),
    (M8, S8, system8),
    (M9, S9, system9),
    (M10, S10, system10),
    (M11, S11, system11),
    (M12, S12, system12)
);
impl_system_collection_for_tuple!(
    (M1, S1, system1),
    (M2, S2, system2),
    (M3, S3, system3),
    (M4, S4, system4),
    (M5, S5, system5),
    (M6, S6, system6),
    (M7, S7, system7),
    (M8, S8, system8),
    (M9, S9, system9),
    (M10, S10, system10),
    (M11, S11, system11),
    (M12, S12, system12),
    (M13, S13, system13)
);
impl_system_collection_for_tuple!(
    (M1, S1, system1),
    (M2, S2, system2),
    (M3, S3, system3),
    (M4, S4, system4),
    (M5, S5, system5),
    (M6, S6, system6),
    (M7, S7, system7),
    (M8, S8, system8),
    (M9, S9, system9),
    (M10, S10, system10),
    (M11, S11, system11),
    (M12, S12, system12),
    (M13, S13, system13),
    (M14, S14, system14)
);
impl_system_collection_for_tuple!(
    (M1, S1, system1),
    (M2, S2, system2),
    (M3, S3, system3),
    (M4, S4, system4),
    (M5, S5, system5),
    (M6, S6, system6),
    (M7, S7, system7),
    (M8, S8, system8),
    (M9, S9, system9),
    (M10, S10, system10),
    (M11, S11, system11),
    (M12, S12, system12),
    (M13, S13, system13),
    (M14, S14, system14),
    (M15, S15, system15)
);
impl_system_collection_for_tuple!(
    (M1, S1, system1),
    (M2, S2, system2),
    (M3, S3, system3),
    (M4, S4, system4),
    (M5, S5, system5),
    (M6, S6, system6),
    (M7, S7, system7),
    (M8, S8, system8),
    (M9, S9, system9),
    (M10, S10, system10),
    (M11, S11, system11),
    (M12, S12, system12),
    (M13, S13, system13),
    (M14, S14, system14),
    (M15, S15, system15),
    (M16, S16, system16)
);

/// When a system runs. [`Stage::Startup`] runs once, after the window and
/// renderer come up (or immediately for an app with no runner). The rest
/// are the per-frame stages, in run order. Game logic belongs in
/// [`Stage::Update`]; [`Stage::LateUpdate`] is for systems that must observe
/// every update system's writes. The `Frame*`, [`Stage::Render`], and
/// [`Stage::Last`] stages carry the engine's own per-frame systems and run
/// after the game stages, where the frame schedule ran historically.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stage {
    Startup,
    First,
    Update,
    LateUpdate,
    FrameStart,
    FrameUpdate,
    FramePostUpdate,
    Render,
    Last,
}

impl Stage {
    const PER_FRAME: [Stage; 8] = [
        Stage::First,
        Stage::Update,
        Stage::LateUpdate,
        Stage::FrameStart,
        Stage::FrameUpdate,
        Stage::FramePostUpdate,
        Stage::Render,
        Stage::Last,
    ];

    const FRAME_STAGES: [Stage; 5] = [
        Stage::FrameStart,
        Stage::FrameUpdate,
        Stage::FramePostUpdate,
        Stage::Render,
        Stage::Last,
    ];

    pub(crate) fn name(self) -> &'static str {
        match self {
            Stage::Startup => "startup",
            Stage::First => "first",
            Stage::Update => "update",
            Stage::LateUpdate => "late_update",
            Stage::FrameStart => "frame_start",
            Stage::FrameUpdate => "frame_update",
            Stage::FramePostUpdate => "frame_post_update",
            Stage::Render => "render",
            Stage::Last => "last",
        }
    }
}

/// Pushes a plain per-frame system into a named frame stage of `stages`,
/// naming it the same way [`App::add_system`] does so the App and
/// runner-owned frame stages register systems identically.
pub(crate) fn push_frame_system<F>(
    stages: &mut nightshade_ecs::Stages<World>,
    stage: Stage,
    system: F,
) where
    F: FnMut(&mut World) + Send + 'static,
{
    stages
        .stage_mut(stage.name())
        .push(std::any::type_name_of_val(&system), system);
}

/// Registers the engine's default per-frame systems into the frame stages
/// of `stages`, which must already declare the [`Stage::FRAME_STAGES`]. Both
/// [`App::new`] and [`build_default_frame_stages`] populate through this, so
/// the App and runner-owned schedules stay in lock-step.
pub(crate) fn register_default_frame_systems(stages: &mut nightshade_ecs::Stages<World>) {
    push_frame_system(
        stages,
        Stage::FrameStart,
        crate::ecs::event::advance_event_frame_system,
    );
    push_frame_system(
        stages,
        Stage::FrameUpdate,
        crate::ecs::graphics::systems::day_night_cycle_system,
    );
    #[cfg(target_arch = "wasm32")]
    push_frame_system(
        stages,
        Stage::FrameUpdate,
        crate::ecs::camera::systems::sync_wasm_canvas_size,
    );
    push_frame_system(
        stages,
        Stage::FrameUpdate,
        crate::ecs::bounding_volume::systems::ensure_bounding_volumes,
    );
    push_frame_system(
        stages,
        Stage::FrameUpdate,
        crate::ecs::animation::systems::update_animation_players,
    );
    push_frame_system(
        stages,
        Stage::FrameUpdate,
        crate::ecs::animation::systems::apply_animations,
    );
    push_frame_system(
        stages,
        Stage::FrameUpdate,
        crate::ecs::tween::update_tweens_system,
    );
    push_frame_system(
        stages,
        Stage::FrameUpdate,
        crate::ecs::particles::systems::update_particle_emitters,
    );
    push_frame_system(
        stages,
        Stage::FrameUpdate,
        crate::ecs::cloth::systems::sync_cloth_meshes_system,
    );
    push_frame_system(
        stages,
        Stage::FrameUpdate,
        crate::ecs::vfx::systems::update_vfx_system,
    );
    push_frame_system(
        stages,
        Stage::FramePostUpdate,
        crate::ecs::transform::systems::run_systems,
    );
    push_frame_system(
        stages,
        Stage::FramePostUpdate,
        crate::ecs::mesh::systems::update_instanced_mesh_caches_system,
    );
    push_frame_system(
        stages,
        Stage::Last,
        crate::ecs::input::systems::apply_ime_allowed_system,
    );
    push_frame_system(
        stages,
        Stage::Last,
        crate::ecs::input::systems::reset_mouse_system,
    );
    push_frame_system(
        stages,
        Stage::Last,
        crate::ecs::input::systems::reset_keyboard_system,
    );
    push_frame_system(
        stages,
        Stage::Last,
        crate::ecs::input::systems::reset_touch_system,
    );
    push_frame_system(
        stages,
        Stage::Last,
        crate::ecs::world::process_commands_system,
    );
    push_frame_system(
        stages,
        Stage::Last,
        crate::ecs::world::cleanup_unused_resources_system,
    );
}

/// Builds a [`nightshade_ecs::Stages`] holding only the engine's frame stages,
/// populated with the default per-frame systems, for runners that drive
/// their own [`State`] loop instead of composing an [`App`]. The stage
/// order matches the tail of [`App`]'s stages, so a runner that runs these
/// after its own logic reproduces the App's per-frame order.
pub fn build_default_frame_stages() -> nightshade_ecs::Stages<World> {
    let mut stages = nightshade_ecs::Stages::new();
    for stage in Stage::FRAME_STAGES {
        stages.add_stage(stage.name());
    }
    register_default_frame_systems(&mut stages);
    stages
}

/// Removes `system` from the frame stages, returning whether anything was
/// removed. Pass the same function value that was registered with
/// [`add_system`](App::add_system); the name is derived from it exactly as
/// registration does, so the caller never spells one out. Runners that pull a
/// system out of their own frame stages (the caller-owned loop lifting the
/// input reset so edge-triggered input survives the frame boundary) call this.
pub fn remove_frame_system<F>(stages: &mut nightshade_ecs::Stages<World>, system: F) -> bool {
    let name = std::any::type_name_of_val(&system);
    let mut removed = false;
    for (_, schedule) in &mut stages.stages {
        removed |= schedule.remove(name);
    }
    removed
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
type RunnerFn = Box<dyn FnOnce(App) -> Result<(), Box<dyn std::error::Error>>>;
type UpdateRenderGraphFn = Box<
    dyn FnMut(
        &mut crate::render::wgpu::rendergraph::RenderGraph<
            crate::render::wgpu::render_configs::RenderInputs,
        >,
        &World,
    ),
>;

/// The plugin-composed application: the world, a startup schedule that
/// runs once, and per-frame [`Stage`]s that plugins push systems into.
///
/// The stages run in [`Stage`] declaration order each frame. The game
/// stages ([`Stage::First`], [`Stage::Update`], [`Stage::LateUpdate`]) run
/// first, then the engine's own per-frame stages ([`Stage::FrameStart`]
/// through [`Stage::Last`]: physics, animation, transforms, retained UI,
/// render sync), exactly where the frame schedule ran historically. Game
/// logic belongs in [`Stage::Update`]; [`Stage::LateUpdate`] is for
/// systems that must observe every update system's writes.
pub struct App {
    pub world: World,
    pub startup: nightshade_ecs::Schedule<World>,
    pub stages: nightshade_ecs::Stages<World>,
    pub(crate) log_guards: Vec<Box<dyn std::any::Any>>,
    render_graph_config: Vec<RenderGraphConfigFn>,
    pre_render: Vec<PreRenderFn>,
    update_render_graph: Vec<UpdateRenderGraphFn>,
    runner: Option<RunnerFn>,
    added_plugins: Vec<std::any::TypeId>,
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl App {
    pub fn new() -> Self {
        let mut stages = nightshade_ecs::Stages::new();
        for stage in Stage::PER_FRAME {
            stages.add_stage(stage.name());
        }
        register_default_frame_systems(&mut stages);
        let mut world = World::default();
        world
            .res_mut::<crate::platform::window::Window>()
            .start_hidden = true;
        world
            .res_mut::<crate::render_driver::Viewport>()
            .create_renderer = false;
        Self {
            world,
            startup: nightshade_ecs::Schedule::new(),
            stages,
            log_guards: Vec::new(),
            render_graph_config: Vec::new(),
            pre_render: Vec::new(),
            update_render_graph: Vec::new(),
            runner: None,
            added_plugins: Vec::new(),
        }
    }

    /// Adds a plugin. Panics if a plugin of the same type was already
    /// added, since building twice would double every system and hook the
    /// plugin registers.
    pub fn add_plugin<P: Plugin + 'static>(mut self, plugin: P) -> Self {
        self.track_plugin(std::any::TypeId::of::<P>(), std::any::type_name::<P>());
        plugin.build(&mut self);
        self
    }

    /// Adds a [`PluginGroup`]'s members in group order, skipping disabled
    /// ones. Members count toward the same once-per-type rule as
    /// [`add_plugin`](App::add_plugin).
    pub fn add_plugins(mut self, group: impl PluginGroup) -> Self {
        for entry in group.build().entries {
            if !entry.enabled {
                continue;
            }
            self.track_plugin(entry.id, entry.name);
            entry.plugin.build(&mut self);
        }
        self
    }

    fn track_plugin(&mut self, id: std::any::TypeId, name: &'static str) {
        if self.added_plugins.contains(&id) {
            panic!("plugin {name} was added twice, each plugin type is added at most once");
        }
        self.added_plugins.push(id);
    }

    /// Registers systems onto a [`Stage`], in registration order. The one
    /// registration method: it takes a plain `fn(&mut World)` system, a
    /// state-and-world `fn(&mut T, &mut World)` game system ([`game_scope`]
    /// applied at registration, state type inferred from the function, the
    /// state inserted with [`App::insert_resource`]), or a tuple mixing
    /// both. [`Stage::Startup`] routes to the startup schedule, which runs
    /// once. Gate a system on an app state by wrapping it in
    /// [`while_in`](crate::states::while_in).
    pub fn add_system<Marker>(
        &mut self,
        stage: Stage,
        systems: impl SystemCollection<Marker>,
    ) -> &mut Self {
        let schedule = match stage {
            Stage::Startup => &mut self.startup,
            per_frame => self.stages.stage_mut(per_frame.name()),
        };
        systems.push_onto(schedule);
        self
    }

    /// Registers a tuple of systems onto a [`Stage`] in one call, in tuple
    /// order, so `app.add_systems(Stage::Update, (input, movement, render))`
    /// stands in for one [`add_system`](App::add_system) per system. Each
    /// tuple member is any shape [`add_system`](App::add_system) accepts, and
    /// the members may mix shapes freely.
    pub fn add_systems<Marker>(
        &mut self,
        stage: Stage,
        systems: impl SystemCollection<Marker>,
    ) -> &mut Self {
        self.add_system(stage, systems)
    }

    /// Removes an engine frame system by passing the same function value it was
    /// registered with, returning whether anything was removed. Drops one of
    /// the default frame systems (the input reset, say) that a custom runner
    /// would rather drive itself.
    pub fn remove_frame_system<F>(&mut self, system: F) -> bool {
        remove_frame_system(&mut self.stages, system)
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

    /// Hands the whole app to a custom runner in place of the built-in
    /// loops: the runner owns the world, the schedules, the render hooks
    /// (through [`App::into_parts`]), and the log guards, and drives them
    /// however its host requires. This is how a plugin claims the main
    /// loop itself, the way `WorkerHostPlugin` hosts the engine inside a
    /// web worker.
    pub fn set_runner(
        &mut self,
        runner: impl FnOnce(App) -> Result<(), Box<dyn std::error::Error>> + 'static,
    ) -> &mut Self {
        self.runner = Some(Box::new(runner));
        self
    }

    /// Runs the app. A runner installed with [`App::set_runner`] takes the
    /// whole app and owns the loop. Otherwise, with neither
    /// [`WindowPlugin`] nor [`RenderPlugin`],
    /// the startup schedule and one pass of the stages run against the
    /// world and the program returns: the do-nothing composition. With
    /// either, the winit event
    /// loop drives the startup schedule once and then the stages every
    /// frame; without [`WindowPlugin`] the window stays hidden, and
    /// without [`RenderPlugin`] no renderer is created.
    pub fn run(mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(runner) = self.runner.take() {
            return runner(self);
        }

        if self
            .world
            .res::<crate::platform::window::Window>()
            .start_hidden
            && !self
                .world
                .res::<crate::render_driver::Viewport>()
                .create_renderer
        {
            let mut world = self.world;
            self.startup.run(&mut world);
            self.stages.run(&mut world);
            return Ok(());
        }

        let log_guards = std::mem::take(&mut self.log_guards);
        let (world, state) = self.into_parts();
        let result = crate::run::launch_with_world(state, world);
        drop(log_guards);
        result
    }

    /// Takes the log appender guards out of the app. A custom runner calls
    /// this before [`App::into_parts`] and keeps the guards alive for the
    /// life of the program so buffered log lines flush.
    pub fn take_log_guards(&mut self) -> Vec<Box<dyn std::any::Any>> {
        std::mem::take(&mut self.log_guards)
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
/// no aliasing and no `unsafe`. Delegates to the nightshade_ecs host scope through
/// the engine `World`'s `ResourceHost` impl: the resource is absent inside
/// the closure and is reinserted even if `body` unwinds. Systems that want
/// several resources at once can import `nightshade_ecs::dynamic::ResourceHostExt`
/// and call `world.resources_scope` instead of nesting.
///
/// Panics if the resource was never inserted or is already scoped.
pub fn game_scope<T: Send + Sync + 'static, R>(
    world: &mut World,
    body: impl FnOnce(&mut T, &mut World) -> R,
) -> R {
    world.resource_scope(|world, value| body(value, world))
}

/// The [`State`] adapter that drives an [`App`]'s schedules through the
/// engine runner: startup once at initialize, stages every frame, render
/// hooks forwarded.
struct StageState {
    startup: nightshade_ecs::Schedule<World>,
    stages: nightshade_ecs::Stages<World>,
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
        app.world
            .res_mut::<crate::platform::window::Window>()
            .title = self.title.clone();
        if self.size.is_some() {
            app.world
                .res_mut::<crate::platform::window::Window>()
                .initial_size = self.size;
        }
        app.world
            .res_mut::<crate::platform::window::Window>()
            .start_hidden = false;
    }
}

/// The standard composition: a visible window, logging, the renderer, and
/// the UI systems. The capability plugins in [`crate::plugins`] stay
/// opt-in: physics because most apps drive it per screen, and audio,
/// gamepad, and egui so a program only acquires the devices and overlays
/// it composes. Customize members at the call site with
/// [`set`](PluginGroup::set) and [`disable`](PluginGroup::disable).
pub struct DefaultPlugins;

impl PluginGroup for DefaultPlugins {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::new()
            .add_plugin(WindowPlugin::default())
            .add_plugin(crate::plugins::log::LogPlugin::default())
            .add_plugin(crate::plugins::render::RenderPlugin::default())
            .add_plugin(crate::plugins::ui::UiPlugin)
    }
}
