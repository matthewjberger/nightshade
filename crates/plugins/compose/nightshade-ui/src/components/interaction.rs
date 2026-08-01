use nalgebra_glm;
use nalgebra_glm::Vec2;

use crate::data::state::{STATE_COUNT, UiBase, UiStateTrait};

#[derive(Clone, Debug, PartialEq)]
pub enum AccessibleRole {
    Button,
    Slider,
    Checkbox,
    Radio,
    Toggle,
    TextInput,
    TextArea,
    Dropdown,
    Tab,
    TabPanel,
    Tree,
    TreeItem,
    Grid,
    GridCell,
    Dialog,
    Alert,
    ProgressBar,
    Menu,
    MenuItem,
}

#[derive(Clone, Debug, Default)]
pub struct UiNodeInteraction {
    pub hovered: bool,
    pub pressed: bool,
    pub clicked: bool,
    pub focused: bool,
    pub drag_start: Option<Vec2>,
    pub dragging: bool,
    pub cursor_icon: Option<nightshade_platform::CursorIcon>,
    pub double_clicked: bool,
    pub right_clicked: bool,
    pub tooltip_text: Option<String>,
    pub tooltip_entity: Option<nightshade::ecs::Entity>,
    pub tab_index: Option<i32>,
    pub disabled: bool,
    pub error_text: Option<String>,
    pub validation_rules: Vec<ValidationRule>,
    pub accessible_role: Option<AccessibleRole>,
    pub accessible_label: Option<String>,
    pub test_id: Option<String>,
}

#[derive(Clone, Copy, Debug)]
pub struct Spring {
    pub stiffness: f32,
    pub damping: f32,
    pub mass: f32,
}

impl Default for Spring {
    fn default() -> Self {
        Self::lively()
    }
}

impl Spring {
    pub fn lively() -> Self {
        Self {
            stiffness: 280.0,
            damping: 22.0,
            mass: 1.0,
        }
    }

    pub fn snappy() -> Self {
        Self {
            stiffness: 420.0,
            damping: 38.0,
            mass: 1.0,
        }
    }

    pub fn gentle() -> Self {
        Self {
            stiffness: 200.0,
            damping: 28.0,
            mass: 1.0,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct StateTransition {
    pub enter_speed: f32,
    pub exit_speed: f32,
    pub easing: crate::data::easing::EasingFunction,
    pub spring: Option<Spring>,
}

impl Default for StateTransition {
    fn default() -> Self {
        Self {
            enter_speed: 10.0,
            exit_speed: 4.0,
            easing: crate::data::easing::EasingFunction::CubicOut,
            spring: None,
        }
    }
}

impl StateTransition {
    pub fn for_state<S: UiStateTrait>() -> Self {
        let index = S::INDEX;
        let hover = crate::data::state::UiHover::INDEX;
        let pressed = crate::data::state::UiPressed::INDEX;
        let focused = crate::data::state::UiFocused::INDEX;
        let easing = if index == hover {
            crate::data::easing::EasingFunction::BackOut
        } else {
            crate::data::easing::EasingFunction::CubicOut
        };
        let spring = if index == hover {
            Some(Spring::lively())
        } else if index == pressed {
            Some(Spring::snappy())
        } else if index == focused {
            Some(Spring::gentle())
        } else {
            None
        };
        Self {
            enter_speed: 10.0,
            exit_speed: 4.0,
            easing,
            spring,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct UiStateWeights {
    pub weights: [f32; STATE_COUNT],
    pub transitions: [Option<StateTransition>; STATE_COUNT],
    pub progress: [f32; STATE_COUNT],
    pub targets: [f32; STATE_COUNT],
    pub start_weights: [f32; STATE_COUNT],
    pub velocity: [f32; STATE_COUNT],
}

impl Default for UiStateWeights {
    fn default() -> Self {
        let mut weights = [0.0; STATE_COUNT];
        weights[UiBase::INDEX] = 1.0;
        let mut targets = [0.0; STATE_COUNT];
        targets[UiBase::INDEX] = 1.0;
        Self {
            weights,
            transitions: [None; STATE_COUNT],
            progress: [1.0; STATE_COUNT],
            targets,
            start_weights: weights,
            velocity: [0.0; STATE_COUNT],
        }
    }
}

#[derive(Clone)]
pub enum ValidationRule {
    Required,
    MinLength(usize),
    MaxLength(usize),
    Custom(fn(&str) -> Result<(), String>),
}

impl std::fmt::Debug for ValidationRule {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Required => write!(formatter, "Required"),
            Self::MinLength(n) => write!(formatter, "MinLength({})", n),
            Self::MaxLength(n) => write!(formatter, "MaxLength({})", n),
            Self::Custom(_) => write!(formatter, "Custom(fn)"),
        }
    }
}

impl ValidationRule {
    pub fn validate(&self, value: &str) -> Result<(), String> {
        match self {
            Self::Required => {
                if value.trim().is_empty() {
                    Err("This field is required".to_string())
                } else {
                    Ok(())
                }
            }
            Self::MinLength(min) => {
                if value.chars().count() < *min {
                    Err(format!("Minimum {} characters required", min))
                } else {
                    Ok(())
                }
            }
            Self::MaxLength(max) => {
                if value.chars().count() > *max {
                    Err(format!("Maximum {} characters allowed", max))
                } else {
                    Ok(())
                }
            }
            Self::Custom(validator) => validator(value),
        }
    }
}
