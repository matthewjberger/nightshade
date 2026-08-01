use crate::resources::CameraRig;
use crate::theme;
use nalgebra_glm::Vec3;

pub fn frame_the_talk(rig: &mut CameraRig, here: Vec3, there: Vec3) {
    rig.target = (here + there) * 0.5 + Vec3::new(0.0, theme::TALK_LIFT, 0.0);
    rig.frame = Some(theme::TALK_DISTANCE);
}

pub fn release_the_frame(rig: &mut CameraRig) {
    rig.frame = None;
}
