pub type UiSystemFn = Box<dyn FnMut(&mut nightshade::ecs::World) + Send>;

pub type SystemFn = UiSystemFn;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UiSystemPhase {
    Interaction,
    PreLayout,
    PreDraw,
}

pub fn ui_system<Marker>(
    system: impl nightshade::ecs::system_param::IntoSystem<nightshade::ecs::World, Marker>,
) -> UiSystemFn {
    Box::new(nightshade::ecs::system_param::into_standalone_runner(
        system,
    ))
}
