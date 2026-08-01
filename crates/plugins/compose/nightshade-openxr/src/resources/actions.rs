use crate::contract::{XrHandState, XrInput};
use nightshade::prelude::{Quat, Vec2, Vec3};
use openxr as xr;

pub struct XrActions {
    action_set: xr::ActionSet,
    move_action: xr::Action<xr::Vector2f>,
    turn_action: xr::Action<xr::Vector2f>,
    left_trigger: xr::Action<f32>,
    right_trigger: xr::Action<f32>,
    left_grip: xr::Action<f32>,
    right_grip: xr::Action<f32>,
    a_button: xr::Action<bool>,
    b_button: xr::Action<bool>,
    left_hand_space: xr::Space,
    right_hand_space: xr::Space,
    left_aim_space: xr::Space,
    right_aim_space: xr::Space,
}

impl XrActions {
    pub fn new(
        instance: &xr::Instance,
        session: &xr::Session<xr::Vulkan>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let action_set = instance.create_action_set("gameplay", "Gameplay", 0)?;

        let move_action = action_set.create_action::<xr::Vector2f>("move", "Move", &[])?;
        let turn_action = action_set.create_action::<xr::Vector2f>("turn", "Turn", &[])?;
        let left_trigger = action_set.create_action::<f32>("left_trigger", "Left Trigger", &[])?;
        let right_trigger =
            action_set.create_action::<f32>("right_trigger", "Right Trigger", &[])?;
        let left_grip = action_set.create_action::<f32>("left_grip", "Left Grip", &[])?;
        let right_grip = action_set.create_action::<f32>("right_grip", "Right Grip", &[])?;
        let a_button = action_set.create_action::<bool>("a_button", "A Button", &[])?;
        let b_button = action_set.create_action::<bool>("b_button", "B Button", &[])?;
        let left_hand = action_set.create_action::<xr::Posef>("left_hand", "Left Hand", &[])?;
        let right_hand = action_set.create_action::<xr::Posef>("right_hand", "Right Hand", &[])?;
        let left_aim = action_set.create_action::<xr::Posef>("left_aim", "Left Aim", &[])?;
        let right_aim = action_set.create_action::<xr::Posef>("right_aim", "Right Aim", &[])?;

        let path = |value: &str| instance.string_to_path(value);

        instance.suggest_interaction_profile_bindings(
            path("/interaction_profiles/oculus/touch_controller")?,
            &[
                xr::Binding::new(&move_action, path("/user/hand/left/input/thumbstick")?),
                xr::Binding::new(&turn_action, path("/user/hand/right/input/thumbstick")?),
                xr::Binding::new(&left_trigger, path("/user/hand/left/input/trigger/value")?),
                xr::Binding::new(
                    &right_trigger,
                    path("/user/hand/right/input/trigger/value")?,
                ),
                xr::Binding::new(&left_grip, path("/user/hand/left/input/squeeze/value")?),
                xr::Binding::new(&right_grip, path("/user/hand/right/input/squeeze/value")?),
                xr::Binding::new(&a_button, path("/user/hand/right/input/a/click")?),
                xr::Binding::new(&b_button, path("/user/hand/right/input/b/click")?),
                xr::Binding::new(&left_hand, path("/user/hand/left/input/grip/pose")?),
                xr::Binding::new(&right_hand, path("/user/hand/right/input/grip/pose")?),
                xr::Binding::new(&left_aim, path("/user/hand/left/input/aim/pose")?),
                xr::Binding::new(&right_aim, path("/user/hand/right/input/aim/pose")?),
            ],
        )?;

        instance.suggest_interaction_profile_bindings(
            path("/interaction_profiles/khr/simple_controller")?,
            &[
                xr::Binding::new(&left_trigger, path("/user/hand/left/input/select/click")?),
                xr::Binding::new(&right_trigger, path("/user/hand/right/input/select/click")?),
                xr::Binding::new(&a_button, path("/user/hand/right/input/menu/click")?),
                xr::Binding::new(&b_button, path("/user/hand/left/input/menu/click")?),
                xr::Binding::new(&left_hand, path("/user/hand/left/input/grip/pose")?),
                xr::Binding::new(&right_hand, path("/user/hand/right/input/grip/pose")?),
                xr::Binding::new(&left_aim, path("/user/hand/left/input/aim/pose")?),
                xr::Binding::new(&right_aim, path("/user/hand/right/input/aim/pose")?),
            ],
        )?;

        session.attach_action_sets(&[&action_set])?;

        let left_hand_space =
            left_hand.create_space(session, xr::Path::NULL, xr::Posef::IDENTITY)?;
        let right_hand_space =
            right_hand.create_space(session, xr::Path::NULL, xr::Posef::IDENTITY)?;
        let left_aim_space = left_aim.create_space(session, xr::Path::NULL, xr::Posef::IDENTITY)?;
        let right_aim_space =
            right_aim.create_space(session, xr::Path::NULL, xr::Posef::IDENTITY)?;

        Ok(Self {
            action_set,
            move_action,
            turn_action,
            left_trigger,
            right_trigger,
            left_grip,
            right_grip,
            a_button,
            b_button,
            left_hand_space,
            right_hand_space,
            left_aim_space,
            right_aim_space,
        })
    }

    pub fn poll(
        &self,
        session: &xr::Session<xr::Vulkan>,
        previous: &XrInput,
    ) -> Result<XrInput, Box<dyn std::error::Error>> {
        session.sync_actions(&[(&self.action_set).into()])?;

        let vector2 = |action: &xr::Action<xr::Vector2f>| -> Vec2 {
            action
                .state(session, xr::Path::NULL)
                .map(|state| nalgebra_glm::vec2(state.current_state.x, state.current_state.y))
                .unwrap_or_else(|_| nalgebra_glm::vec2(0.0, 0.0))
        };
        let scalar = |action: &xr::Action<f32>| -> f32 {
            action
                .state(session, xr::Path::NULL)
                .map(|state| state.current_state)
                .unwrap_or(0.0)
        };
        let boolean = |action: &xr::Action<bool>| -> bool {
            action
                .state(session, xr::Path::NULL)
                .map(|state| state.current_state)
                .unwrap_or(false)
        };

        Ok(XrInput {
            left: XrHandState {
                thumbstick: vector2(&self.move_action),
                trigger: scalar(&self.left_trigger),
                grip: scalar(&self.left_grip),
                tracked: false,
                aim: None,
            },
            right: XrHandState {
                thumbstick: vector2(&self.turn_action),
                trigger: scalar(&self.right_trigger),
                grip: scalar(&self.right_grip),
                tracked: false,
                aim: None,
            },
            a_button: boolean(&self.a_button),
            b_button: boolean(&self.b_button),
            previous_a_button: previous.a_button,
            previous_b_button: previous.b_button,
        })
    }

    pub fn hand_pose(
        &self,
        hand: crate::contract::XrHand,
        stage: &xr::Space,
        display_time: xr::Time,
    ) -> Option<(Vec3, Quat)> {
        let space = match hand {
            crate::contract::XrHand::Left => &self.left_hand_space,
            crate::contract::XrHand::Right => &self.right_hand_space,
        };
        locate_pose(space, stage, display_time)
    }

    pub fn aim_pose(
        &self,
        hand: crate::contract::XrHand,
        stage: &xr::Space,
        display_time: xr::Time,
    ) -> Option<(Vec3, Quat)> {
        let space = match hand {
            crate::contract::XrHand::Left => &self.left_aim_space,
            crate::contract::XrHand::Right => &self.right_aim_space,
        };
        locate_pose(space, stage, display_time)
    }
}

fn locate_pose(
    space: &xr::Space,
    stage: &xr::Space,
    display_time: xr::Time,
) -> Option<(Vec3, Quat)> {
    let location = space.locate(stage, display_time).ok()?;
    let required =
        xr::SpaceLocationFlags::POSITION_VALID | xr::SpaceLocationFlags::ORIENTATION_VALID;
    if !location.location_flags.contains(required) {
        return None;
    }
    Some((
        crate::data::to_vec3(location.pose.position),
        crate::data::to_quat(location.pose.orientation),
    ))
}
