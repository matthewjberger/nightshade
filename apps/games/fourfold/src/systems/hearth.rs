use crate::commands as game;
use crate::components::Extent;
use crate::contract::{Cue, Dimension};
use crate::data;
use crate::queries;
use crate::resources::FourfoldResources;
use nightshade::prelude::*;
use nightshade_graphics::commands::builtins::set_mesh_material;
use nightshade_platform::prelude::Time;
use nightshade_scene::prelude::LocalTransform;

pub fn walk(
    time: Res<Time>,
    mut resources: ResMut<FourfoldResources>,
    blocks: Query<(&Dimension, &Extent, &LocalTransform)>,
) {
    let delta_time = time.delta_time.min(0.05);
    let solids = queries::gather_solids(blocks, Dimension::Hearth);
    let drive = resources.drive;
    let villager = &mut resources.villager;

    let wish = Vec3::new(drive.wish.x, 0.0, -drive.wish.y);
    let paced = if wish.norm() > 1.0e-4 {
        wish.normalize() * data::HEARTH_SPEED
    } else {
        Vec3::zeros()
    };
    villager.velocity.x = paced.x;
    villager.velocity.z = paced.z;
    villager.velocity.y =
        (villager.velocity.y - data::GRAVITY * delta_time).max(-data::TERMINAL_FALL);

    let (position, velocity, grounded) =
        queries::slide(villager.position, villager.velocity, &solids, delta_time);
    villager.position = position;
    villager.velocity = velocity;
    villager.grounded = grounded;
    if paced.norm() > 1.0e-4 {
        villager.yaw = (-paced.x).atan2(-paced.z);
    }
    if villager.position.y < data::HUB_FLOOR {
        villager.position = data::HEARTH_START;
        villager.velocity = Vec3::zeros();
    }
    villager.focus +=
        (villager.position - villager.focus) * (data::GLASS_CATCH * delta_time).clamp(0.0, 1.0);
}

pub fn draw(resources: Res<FourfoldResources>, mut commands: Commands) {
    if resources.figure == Entity::default() {
        return;
    }
    commands.set(
        resources.figure,
        LocalTransform {
            translation: resources.villager.position + Vec3::new(0.0, 0.9, 0.0),
            rotation: nalgebra_glm::quat_angle_axis(resources.villager.yaw, &Vec3::y()),
            scale: Vec3::new(0.7, 1.8, 0.7),
        },
    );
}

pub fn reach(mut resources: ResMut<FourfoldResources>, mut commands: Commands) {
    let index = queries::tone_index(Dimension::Hearth);
    if resources.lit[index] {
        return;
    }
    let flat = Vec3::new(
        resources.villager.position.x - data::HEARTH_WELL.x,
        0.0,
        resources.villager.position.z - data::HEARTH_WELL.z,
    );
    if flat.norm() > data::HEARTH_REACH {
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
