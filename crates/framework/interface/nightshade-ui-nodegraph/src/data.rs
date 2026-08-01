use nalgebra_glm::Vec2;
use nightshade::ecs::Entity;

#[derive(Default, Clone, Copy, Debug, PartialEq, Eq)]
pub struct Socket {
    pub node: Entity,
    pub index: u32,
}

#[derive(Default, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Holding {
    #[default]
    Nothing,
    View,
    Node(Entity),
    Note(Entity),
    Inlet(Socket),
    Outlet(Socket),
}

#[derive(Clone, Copy, Debug)]
pub struct NodesLinked {
    pub from: Socket,
    pub to: Socket,
}

#[derive(Clone, Copy, Debug)]
pub struct NodesUnlinked {
    pub from: Socket,
    pub to: Socket,
}

pub const WIRE_STEPS: usize = 24;

pub fn wire_grips(from: Vec2, to: Vec2, reach: f32) -> (Vec2, Vec2) {
    let span = (to.x - from.x).abs().max(reach);
    let pull = span.min(reach * 4.0);
    (
        Vec2::new(from.x + pull, from.y),
        Vec2::new(to.x - pull, to.y),
    )
}

pub fn wire_point(from: Vec2, first: Vec2, second: Vec2, to: Vec2, turn: f32) -> Vec2 {
    let rest = 1.0 - turn;
    from * (rest * rest * rest)
        + first * (3.0 * rest * rest * turn)
        + second * (3.0 * rest * turn * turn)
        + to * (turn * turn * turn)
}

pub fn step_distance(point: Vec2, first: Vec2, second: Vec2) -> f32 {
    let along = second - first;
    let length = along.magnitude_squared();
    if length <= f32::EPSILON {
        return (point - first).magnitude();
    }
    let share = ((point - first).dot(&along) / length).clamp(0.0, 1.0);
    (point - (first + along * share)).magnitude()
}
