use crate::commands::dungeon::teleport;
use crate::commands::interior::spawn_room;
use crate::queries::camera::eye_offset;
use crate::queries::interior::landing_spot;
use crate::queries::terrain::height_at;
use crate::resources::{Diorama, Home, Rig, Sheet, Terrain};
use crate::tags::Indoors;
use crate::theme;
use nalgebra_glm::{Vec2, Vec3};
use nightshade::prelude::*;

pub fn build_the_room(
    mut state: ResMut<Home>,
    mut diorama: ResMut<Diorama>,
    sheet: Res<Sheet>,
    terrain: Res<Terrain>,
    mut rig: ResMut<Rig>,
    mut commands: Commands,
    inside: Query<(&nightshade_scene::prelude::LocalTransform,), WithTag<Indoors>>,
) {
    let (pending, leaving) = (state.pending, state.leaving);
    if pending.is_none() && !leaving {
        return;
    }
    let mut old = Vec::new();
    inside.for_each(|entity, _| old.push(entity));
    for entity in old {
        commands.despawn_recursive(entity);
    }
    let farmer = diorama.farmer;
    let camera = diorama.camera;
    let sheet = *sheet;
    let mut home = std::mem::take(&mut *state);
    home.leaving = false;
    home.pending = None;
    let landing = match pending {
        Some(kind) if !leaving => {
            spawn_room(&mut commands, &sheet, &mut home, kind);
            home.active = true;
            let spot = landing_spot(&home);
            Vec3::new(spot.x, theme::INDOOR_Y, spot.y)
        }
        _ => {
            let left = home.house;
            home.active = false;
            home.house = None;
            let spot = left.map_or(theme::FARMER_START, |kind| {
                kind.door_spot() + Vec2::new(0.0, theme::DOOR_STEP)
            });
            let height = height_at(&terrain, spot.x, spot.y).unwrap_or(theme::VILLAGE_HEIGHT);
            Vec3::new(spot.x, height as f32, spot.y)
        }
    };
    *state = home;
    if let Some(farmer) = farmer {
        teleport(&mut commands, farmer, landing);
    }
    if let Some(camera) = camera {
        rig.focus = landing;
        let eye = landing + eye_offset(rig.yaw);
        teleport(&mut commands, camera, eye);
    }
    diorama.feet = landing;
}
