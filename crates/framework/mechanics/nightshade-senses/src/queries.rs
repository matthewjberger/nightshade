use crate::components::{Alert, Exposure, Hearing, LastSeen, Sight};
use crate::data::Noise;
use crate::resources::{Noises, SensesSettings};
use nalgebra_glm::{Quat, Vec3, quat_rotate_vec3};
use nightshade_arc::in_arc;
use nightshade_plane::queries::across;

pub fn facing(rotation: &Quat) -> Vec3 {
    quat_rotate_vec3(rotation, &Vec3::new(0.0, 0.0, -1.0))
}

pub fn loudest(
    noises: &Noises,
    settings: &SensesSettings,
    at: Vec3,
    hearing: &Hearing,
) -> Option<Noise> {
    let mut best: Option<Noise> = None;
    for noise in &noises.list {
        let reach = hearing.0 * noise.loudness;
        if across(settings.plane, at, noise.at) > reach {
            continue;
        }
        if best.is_none_or(|held| noise.loudness > held.loudness) {
            best = Some(*noise);
        }
    }
    best
}

pub fn visible(
    settings: &SensesSettings,
    from: Vec3,
    rotation: &Quat,
    target: Vec3,
    sight: &Sight,
    exposure: &Exposure,
) -> bool {
    in_arc(
        settings.plane,
        from,
        facing(rotation),
        target,
        sight.0 * exposure.0,
        settings.half_angle,
    )
}

pub fn alertness(alert: &Alert) -> f32 {
    alert.0
}

pub fn alarmed(alert: &Alert, settings: &SensesSettings) -> bool {
    alert.0 >= settings.alarmed_above
}

pub fn last_seen(seen: &LastSeen) -> Vec3 {
    seen.0
}
