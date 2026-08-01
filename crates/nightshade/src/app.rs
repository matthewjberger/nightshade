use nightshade_ecs::World;

pub trait SystemCollection<Marker> {
    fn push_onto(self, schedule: &mut nightshade_ecs::Schedule<World>);
}

pub struct WorldSystemMarker;

impl<F> SystemCollection<WorldSystemMarker> for F
where
    F: FnMut(&mut World) + Send + 'static,
{
    fn push_onto(self, schedule: &mut nightshade_ecs::Schedule<World>) {
        schedule.push(std::any::type_name_of_val(&self), self);
    }
}

pub struct ParamSystemMarker<Marker>(std::marker::PhantomData<Marker>);

impl<Marker, F> SystemCollection<ParamSystemMarker<Marker>> for F
where
    F: nightshade_ecs::system_param::IntoSystem<World, Marker>,
{
    fn push_onto(self, schedule: &mut nightshade_ecs::Schedule<World>) {
        schedule.push_with_access(
            std::any::type_name_of_val(&self),
            F::access(),
            F::warmer(),
            F::flusher(),
            self.into_runner(),
        );
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

    pub fn name(self) -> &'static str {
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

pub fn remove_frame_system<F>(stages: &mut nightshade_ecs::Stages<World>, system: F) -> bool {
    let name = std::any::type_name_of_val(&system);
    let mut removed = false;
    for (_, schedule) in &mut stages.stages {
        removed |= schedule.remove(name);
    }
    removed
}

type RunnerFn = Box<dyn FnOnce(App) -> Result<(), Box<dyn std::error::Error>>>;

#[cfg(debug_assertions)]
pub fn report_batch_layout(stages: &nightshade_ecs::Stages<World>) {
    for (name, schedule) in &stages.stages {
        tracing::debug!(
            target: "nightshade::stages",
            "{name}: {}",
            schedule.batch_layout()
        );
    }
}

#[cfg(not(debug_assertions))]
pub fn report_batch_layout(_stages: &nightshade_ecs::Stages<World>) {}

pub fn push_frame_system<Marker>(
    stages: &mut nightshade_ecs::Stages<World>,
    stage: Stage,
    system: impl SystemCollection<Marker>,
) {
    let schedule = stages.stage_mut(stage.name());
    let name = std::any::type_name_of_val(&system);
    assert!(
        !schedule.contains(name),
        "frame system \"{name}\" is registered twice; a plugin registers its systems once, and `App::add_system` is the path that numbers a repeat"
    );
    system.push_onto(schedule);
}

pub struct Need {
    pub id: std::any::TypeId,
    pub what: &'static str,
    pub from: &'static str,
}

impl Need {
    pub fn resource<T: Send + Sync + 'static>(from: &'static str) -> Self {
        Self {
            id: std::any::TypeId::of::<T>(),
            what: std::any::type_name::<T>(),
            from,
        }
    }
}

#[macro_export]
macro_rules! needs {
    ($($resource:ty => $from:literal),* $(,)?) => {
        vec![$($crate::app::Need::resource::<$resource>($from)),*]
    };
}

pub struct App {
    pub world: World,
    pub startup: nightshade_ecs::Schedule<World>,
    pub stages: nightshade_ecs::Stages<World>,
    kept_alive: Vec<Box<dyn std::any::Any>>,
    runner: Option<(&'static str, RunnerFn)>,
    pub(crate) added_plugins: Vec<std::any::TypeId>,
    pub(crate) declared_needs: Vec<(&'static str, Need)>,
    plugin_part: Option<Box<dyn std::any::Any>>,
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl App {
    fn spent() -> Self {
        Self {
            world: nightshade_ecs::World::tracking(),
            startup: nightshade_ecs::Schedule::new(),
            stages: nightshade_ecs::Stages::new(),
            kept_alive: Vec::new(),
            runner: None,
            added_plugins: Vec::new(),
            declared_needs: Vec::new(),
            plugin_part: None,
        }
    }

    pub fn new() -> Self {
        let mut stages = nightshade_ecs::Stages::new();
        for stage in Stage::PER_FRAME {
            stages.add_stage(stage.name());
        }
        let mut world = nightshade_ecs::World::tracking();
        world.insert_resource(nightshade_ecs::HierarchyIndex::default());
        Self {
            world,
            startup: nightshade_ecs::Schedule::new(),
            stages,
            kept_alive: Vec::new(),
            runner: None,
            added_plugins: Vec::new(),
            declared_needs: Vec::new(),
            plugin_part: None,
        }
    }

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

    pub fn remove_frame_system<F>(&mut self, system: F) -> bool {
        remove_frame_system(&mut self.stages, system)
    }

    pub fn insert_resource<T: Send + Sync + 'static>(&mut self, value: T) -> &mut Self {
        self.world.insert_resource(value);
        self
    }

    pub fn set_runner(
        &mut self,
        claimant: &'static str,
        runner: impl FnOnce(App) -> Result<(), Box<dyn std::error::Error>> + 'static,
    ) -> &mut Self {
        if let Some((held_by, _)) = &self.runner {
            panic!(
                "{claimant} wants the main loop, which {held_by} already holds. Compose one of the two, or call take_over_runner to replace it on purpose."
            );
        }
        self.runner = Some((claimant, Box::new(runner)));
        self
    }

    pub fn take_over_runner(
        &mut self,
        claimant: &'static str,
        runner: impl FnOnce(App) -> Result<(), Box<dyn std::error::Error>> + 'static,
    ) -> &mut Self {
        self.runner = Some((claimant, Box::new(runner)));
        self
    }

    pub fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut app = std::mem::replace(self, Self::spent());
        app.check_the_needs();
        if let Some((_, runner)) = app.runner.take() {
            return runner(app);
        }

        app.stages.apply_orderings();
        report_batch_layout(&app.stages);
        let mut world = app.world;
        app.startup.run(&mut world);
        app.stages.run(&mut world);
        Ok(())
    }

    pub fn plugin_part(&mut self) -> &mut Option<Box<dyn std::any::Any>> {
        &mut self.plugin_part
    }

    pub fn keep_alive<T: 'static>(&mut self, value: T) -> &mut Self {
        self.kept_alive.push(Box::new(value));
        self
    }

    pub fn take_kept_alive(&mut self) -> Vec<Box<dyn std::any::Any>> {
        std::mem::take(&mut self.kept_alive)
    }
    pub fn check_the_needs(&self) {
        let mut missing = Vec::new();
        for (plugin, need) in &self.declared_needs {
            if self.world.resources.holds(need.id) {
                continue;
            }
            missing.push(format!(
                "  {plugin} needs {}, which {} installs",
                need.what, need.from
            ));
        }
        if missing.is_empty() {
            return;
        }
        panic!(
            "this program composes plugins whose needs are not met:
{}
compose the plugins named, or take the plugins that need them out",
            missing.join(
                "
"
            )
        );
    }

    pub fn hold_needs(&mut self, plugin: &'static str, needs: Vec<Need>) {
        for need in needs {
            self.declared_needs.push((plugin, need));
        }
    }

    pub fn into_schedules(
        mut self,
    ) -> (
        World,
        nightshade_ecs::Schedule<World>,
        nightshade_ecs::Stages<World>,
    ) {
        self.check_the_needs();
        self.stages.apply_orderings();
        report_batch_layout(&self.stages);
        let App {
            world,
            startup,
            stages,
            ..
        } = self;
        (world, startup, stages)
    }
}
