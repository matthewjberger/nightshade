use crate::commands as game;
use crate::components::Cell;
use crate::contract::{Cue, Dimension};
use crate::data;
use crate::resources::FourfoldResources;
use nightshade::prelude::*;
use nightshade_graphics::commands::builtins::set_mesh_material;
use nightshade_platform::prelude::Time;
use nightshade_scene::prelude::LocalTransform;

pub fn step(
    time: Res<Time>,
    mut resources: ResMut<FourfoldResources>,
    mut blocks: Query<(&mut Cell, &mut LocalTransform)>,
) {
    let delta_time = time.delta_time.min(0.05);
    let wish = resources.drive.wish;
    let stepper = &mut resources.stepper;
    stepper.rest = (stepper.rest - delta_time).max(0.0);
    if stepper.rest > 0.0 {
        return;
    }

    let heading = if wish.y > 0.5 {
        [0, -1]
    } else if wish.y < -0.5 {
        [0, 1]
    } else if wish.x > 0.5 {
        [1, 0]
    } else if wish.x < -0.5 {
        [-1, 0]
    } else {
        return;
    };

    let ahead = [stepper.cell[0] + heading[0], stepper.cell[1] + heading[1]];
    if !data::loom_inside(ahead) {
        return;
    }

    let beyond = [ahead[0] + heading[0], ahead[1] + heading[1]];
    let mut shoved = None;
    let mut blocked = false;
    blocks.for_each_mut(|entity, (cell, _transform)| {
        if cell.0 == ahead {
            shoved = Some(entity);
        }
        if cell.0 == beyond {
            blocked = true;
        }
    });

    if let Some(entity) = shoved {
        if !data::loom_inside(beyond) || blocked {
            return;
        }
        blocks.for_entity(entity, |(cell, transform)| {
            cell.0 = beyond;
            transform.translation = data::loom_cell(beyond) + Vec3::new(0.0, 0.7, 0.0);
        });
    }

    stepper.cell = ahead;
    stepper.rest = data::LOOM_STEP_SECONDS;
}

pub fn settle(
    mut resources: ResMut<FourfoldResources>,
    mut commands: Commands,
    mut blocks: Query<(&Cell, &LocalTransform)>,
) {
    let index = crate::queries::tone_index(Dimension::Loom);
    if resources.lit[index] {
        return;
    }
    let mut resting = 0;
    blocks.for_each_mut(|_entity, (cell, _transform)| {
        if data::LOOM_PLATES.contains(&cell.0) {
            resting += 1;
        }
    });
    if resting < data::LOOM_PLATES.len() {
        return;
    }
    resources.lit[index] = true;
    game::queue_cue(&mut resources.story, Cue::Solved);
    if resources.frames[index] != Entity::default() {
        set_mesh_material(
            &mut commands,
            resources.frames[index],
            data::TONES[index].material,
        );
    }
}

pub fn carry(resources: Res<FourfoldResources>, mut commands: Commands) {
    if resources.pawn == Entity::default() {
        return;
    }
    commands.set(
        resources.pawn,
        LocalTransform {
            translation: data::loom_cell(resources.stepper.cell) + Vec3::new(0.0, 0.8, 0.0),
            rotation: Quat::identity(),
            scale: Vec3::new(0.9, 1.6, 0.9),
        },
    );
}
