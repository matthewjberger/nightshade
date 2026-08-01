use nalgebra_glm::Vec2;

#[derive(Clone, Debug, PartialEq)]
pub enum NeonStaticEvent {
    ShotFired { at: Vec2 },
    EnemyKilled { at: Vec2 },
    PlayerKilled { at: Vec2 },
}
