use crate::commands::{spawn_bar, spawn_block, spawn_spike, spawn_stone};
use crate::data::{Prop, Settings};
use crate::resources::SdfScene;
use nalgebra_glm::Vec3;
use nightshade::prelude::*;
use nightshade_config::prelude::Config;
use nightshade_sdf::data::surface::chunk_of;
use nightshade_sdf::resources::SdfCollision;

pub fn release(
    mut scene: ResMut<SdfScene>,
    collision: Res<SdfCollision>,
    config: Res<Config<Settings>>,
    mut commands: Commands,
) {
    if scene.waiting.is_empty() {
        return;
    }
    let cell = collision.cell;
    let reach = Vec3::new(0.0, cell, 0.0);
    let waiting = std::mem::take(&mut scene.waiting);
    for held in waiting {
        let grounded = [held.ground, held.ground - reach, held.ground + reach]
            .iter()
            .any(|point| scene.ground.contains_key(&chunk_of(point, cell)));
        if !grounded {
            scene.waiting.push(held);
            continue;
        }
        commands.despawn_recursive(held.anchor);
        match held.prop {
            Prop::Bar => {
                spawn_bar(&mut commands, &config, held.spot, held.rotation);
            }
            Prop::Block => {
                spawn_block(&mut commands, &config, held.spot, held.rotation);
            }
            Prop::Spike => {
                spawn_spike(&mut commands, &config, held.spot, held.rotation);
            }
            Prop::Stone => {
                let stone = spawn_stone(&mut commands, &config, held.spot, Vec3::zeros());
                scene.stones.push_back(stone);
            }
        }
    }
}
