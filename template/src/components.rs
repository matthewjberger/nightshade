/// A game component: entities carrying it spin about the Y axis at `speed`
/// radians per second. Game components live on engine entities in the app
/// member world; set one with `world.set(entity, Spin { .. })` and the engine
/// registers the type on first use, no schema to declare.
#[derive(Default, Clone, Debug)]
pub struct Spin {
    pub speed: f32,
}
