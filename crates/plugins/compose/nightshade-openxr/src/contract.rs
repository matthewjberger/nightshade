use nalgebra_glm::{Quat, Vec2, Vec3};
use nightshade_ecs::Entity;

#[derive(Debug, Clone, Copy)]
pub struct XrSettings {
    pub z_near: f32,
    pub eye_resolution: Option<(u32, u32)>,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct XrRigSlot(pub Option<XrRig>);

#[derive(Debug, Clone, Copy)]
pub struct XrRig {
    pub origin: Entity,
    pub left_eye: Entity,
    pub right_eye: Entity,
    pub head: Entity,
    pub left_hand: Entity,
    pub right_hand: Entity,
    pub pointer: Entity,
}

impl XrRig {
    pub fn eye(&self, eye: XrEye) -> Entity {
        match eye {
            XrEye::Left => self.left_eye,
            XrEye::Right => self.right_eye,
        }
    }

    pub fn hand(&self, hand: XrHand) -> Entity {
        match hand {
            XrHand::Left => self.left_hand,
            XrHand::Right => self.right_hand,
        }
    }

    pub fn hand_state(input: &XrInput, hand: XrHand) -> XrHandState {
        match hand {
            XrHand::Left => input.left,
            XrHand::Right => input.right,
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct XrView {
    pub position: Vec3,
    pub orientation: Quat,
    pub tangents: [f32; 4],
}

#[derive(Debug, Clone, Default)]
pub struct XrFrame {
    pub views: Vec<XrView>,
    pub head_position: Vec3,
    pub head_orientation: Quat,
    pub tracking_valid: bool,
    pub delta_time: f32,
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct XrPose {
    pub position: Vec3,
    pub orientation: Quat,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct XrHandState {
    pub thumbstick: Vec2,
    pub trigger: f32,
    pub grip: f32,
    pub tracked: bool,
    pub aim: Option<XrPose>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum XrUiAnchor {
    Head,
    Stage(XrPose),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct XrUiPanel {
    pub enabled: bool,
    pub anchor: XrUiAnchor,
    pub distance: f32,
    pub width: f32,
    pub pointer_hand: XrHand,
}

impl Default for XrUiPanel {
    fn default() -> Self {
        Self {
            enabled: true,
            anchor: XrUiAnchor::Head,
            distance: 1.0,
            width: 1.0,
            pointer_hand: XrHand::Right,
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct XrUiPointer {
    pub hit: Option<Vec2>,
    pub hit_world: Option<Vec3>,
    pub pressed: bool,
}

impl XrHandState {
    pub fn trigger_pressed(&self) -> bool {
        self.trigger > 0.5
    }

    pub fn grip_pressed(&self) -> bool {
        self.grip > 0.5
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct XrInput {
    pub left: XrHandState,
    pub right: XrHandState,
    pub a_button: bool,
    pub b_button: bool,
    pub previous_a_button: bool,
    pub previous_b_button: bool,
}

impl XrInput {
    pub fn a_pressed(&self) -> bool {
        self.a_button && !self.previous_a_button
    }

    pub fn b_pressed(&self) -> bool {
        self.b_button && !self.previous_b_button
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum XrEye {
    Left,
    Right,
}

impl XrEye {
    pub const ALL: [XrEye; 2] = [XrEye::Left, XrEye::Right];

    pub fn layer(self) -> u32 {
        match self {
            XrEye::Left => 0,
            XrEye::Right => 1,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum XrHand {
    Left,
    Right,
}

impl XrHand {
    pub const ALL: [XrHand; 2] = [XrHand::Left, XrHand::Right];
}
