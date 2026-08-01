use crate::components::DragPayload;
use nalgebra_glm;
use nalgebra_glm::Vec2;

#[derive(Clone, Debug)]
pub struct ActiveDrag {
    pub source: nightshade::ecs::Entity,
    pub payload: DragPayload,
    pub start_position: Vec2,
    pub current_position: Vec2,
}

#[derive(Default, Debug)]
pub struct RetainedUiDragDrop {
    pub active: Option<ActiveDrag>,
    pub over_entity: Option<nightshade::ecs::Entity>,
}

#[derive(Default, Debug)]
pub struct RetainedUiInteraction {
    pub focused_entity: Option<nightshade::ecs::Entity>,
    pub hovered_entity: Option<nightshade::ecs::Entity>,
    pub active_entity: Option<nightshade::ecs::Entity>,
    pub last_click: Option<(nightshade::ecs::Entity, f64, Vec2)>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum NavDirection {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug)]
pub struct RetainedUiGamepadNav {
    pub held_direction: Option<NavDirection>,
    pub next_repeat_at: f64,
    pub deadzone: f32,
    pub initial_repeat_delay: f64,
    pub repeat_interval: f64,
    pub enabled: bool,
}

impl Default for RetainedUiGamepadNav {
    fn default() -> Self {
        Self {
            held_direction: None,
            next_repeat_at: 0.0,
            deadzone: 0.5,
            initial_repeat_delay: 0.4,
            repeat_interval: 0.12,
            enabled: true,
        }
    }
}

#[derive(Clone, Debug)]
pub struct TooltipActiveState {
    pub entity: nightshade::ecs::Entity,
    pub hover_start: f64,
    pub text: String,
    pub tooltip_entity: Option<nightshade::ecs::Entity>,
}
