use crate::resources::Hero;
use crate::theme;
use nalgebra_glm::Vec4;
use nightshade::ecs::QueryTuple;
use nightshade::ecs::system_param::{Query, QueryFilter};
use nightshade_text::prelude::*;

pub fn plate_properties() -> TextProperties {
    TextProperties {
        font_size: theme::PLATE_SIZE,
        color: Vec4::new(
            theme::PLATE_COLOR[0],
            theme::PLATE_COLOR[1],
            theme::PLATE_COLOR[2],
            theme::PLATE_COLOR[3],
        ),
        alignment: TextAlignment::Center,
        vertical_alignment: VerticalAlignment::Middle,
        outline_width: 1.5,
        outline_color: Vec4::new(0.0, 0.0, 0.0, 1.0),
        ..Default::default()
    }
}

pub fn joint_of(name: &str) -> Option<crate::data::joints::Joint> {
    use crate::data::joints::Joint;
    match name {
        "leg-left" => Some(Joint::LegLeft),
        "leg-right" => Some(Joint::LegRight),
        "arm-left" => Some(Joint::ArmLeft),
        "arm-right" => Some(Joint::ArmRight),
        _ => None,
    }
}

pub fn soaring_pose(joint: crate::data::joints::Joint, gait: f32) -> nalgebra_glm::Quat {
    let side = if joint.left() { 1.0 } else { -1.0 };
    let (lift, spread) = if joint.arm() {
        (theme::SOAR_ARM_LIFT, -side * theme::SOAR_ARM_SPREAD)
    } else {
        (
            theme::SOAR_LEG_BACK + (gait + joint.offset()).sin() * theme::SOAR_LEG_WAVE,
            side * theme::SOAR_LEG_SPLAY,
        )
    };
    nalgebra_glm::quat_angle_axis(spread, &nalgebra_glm::Vec3::z())
        * nalgebra_glm::quat_angle_axis(lift, &nalgebra_glm::Vec3::x())
}

pub fn head_node(name: &str) -> bool {
    name == "head"
}

pub fn angle_delta(from: f32, to: f32) -> f32 {
    let mut delta = (to - from) % std::f32::consts::TAU;
    if delta > std::f32::consts::PI {
        delta -= std::f32::consts::TAU;
    }
    if delta < -std::f32::consts::PI {
        delta += std::f32::consts::TAU;
    }
    delta
}

pub fn plate_above(place: nalgebra_glm::Vec3) -> nalgebra_glm::Vec3 {
    place + nalgebra_glm::Vec3::new(0.0, theme::PLATE_HEIGHT, 0.0)
}

pub fn looks_at(
    eye: nalgebra_glm::Vec3,
    forward: nalgebra_glm::Vec3,
    point: nalgebra_glm::Vec3,
    reach: f32,
    cosine: f32,
) -> bool {
    let offset = point - eye;
    let distance = offset.norm();
    distance > 0.01 && distance <= reach && offset.dot(&forward) / distance >= cosine
}

pub fn read_the_hero<Q: QueryTuple, F: QueryFilter, T>(
    hero: &Hero,
    query: &mut Query<'_, Q, F>,
    read: impl for<'item> FnOnce(Q::Item<'item>) -> T,
) -> Option<T> {
    let entity = hero.0?;
    let mut found = None;
    query.for_entity(entity, |item| found = Some(read(item)));
    found
}
