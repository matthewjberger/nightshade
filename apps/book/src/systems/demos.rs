use nightshade_platform::contract::time::Time;

use crate::commands::scene;
use crate::data::demo::DemoEntry;
use crate::resources::PageStage;
use nightshade::prelude::*;

macro_rules! demos {
    ($($module:ident => $title:literal, $blurb:literal;)*) => {
        $(pub mod $module;)*

        pub const DEMOS: &[DemoEntry] = &[
            $(DemoEntry {
                id: stringify!($module),
                title: $title,
                blurb: $blurb,
                mount: $module::mount,
                step: $module::step,
                controls: $module::CONTROLS,
            },)*
        ];
    };
}

demos! {
    spinning_triangle => "A spinning triangle", "Three vertices, one material, one camera";
    orbiting_cubes => "Cubes on an orbit", "A parent turns and the children go with it";
    light_rig => "One light around a sphere", "One light moved around a sphere";
    material_sampler => "Roughness and metal", "Roughness and metal on five spheres";
    picking_table => "Pick a shape", "A ray from the pointer into the scene";
    transform_parts => "Moving one cube", "The three parts a transform is made of";
    hierarchy => "A cube and what hangs off it", "Children follow the parent they are attached to";
    spawn_flock => "A shell of cubes", "Rows spawned and despawned while one pass reads them";
    plugin_stack => "Three behaviors over one ring", "Each switched on and off on its own";
    sky_dial => "Skies and the hour", "The sky is a setting, not a thing in the world";
    emissive_row => "Light off a surface", "Emission, bloom and exposure on five balls";
}

pub fn demo_entry(id: &str) -> Option<&'static DemoEntry> {
    DEMOS.iter().find(|entry| entry.id == id)
}

pub fn apply(
    stage: Res<PageStage>,
    mut scene_state: crate::resources::DemoScene,
    storage: &mut nightshade::ecs::Storage,
) {
    let scene = &mut scene_state;
    let wanted: Vec<&'static str> = stage.figures.iter().map(|figure| figure.demo).collect();
    let active = scene.runtime.active.clone();
    if wanted == active {
        return;
    }

    for demo in &active {
        if !wanted.contains(demo) {
            scene::end(storage, scene, demo);
        }
    }

    let still_up = !scene.runtime.active.is_empty();
    if wanted.is_empty() && !still_up {
        scene::rest(scene);
        return;
    }

    for demo in &wanted {
        if scene.runtime.active.contains(demo) {
            continue;
        }
        let Some(entry) = demo_entry(demo) else {
            continue;
        };
        let first = scene.runtime.active.is_empty();
        scene::begin(scene, demo, first);
        (entry.mount)(storage, scene);
        scene.runtime.active.push(demo);
        scene.runtime.mounting = None;
    }

    if scene.runtime.active.is_empty() {
        scene::rest(scene);
    }
}

pub fn advance(
    time: Res<Time>,
    mut scene_state: crate::resources::DemoScene,
    storage: &mut nightshade::ecs::Storage,
) {
    let scene = &mut scene_state;
    let delta_time = time.delta_time;
    let active = scene.runtime.active.clone();

    for demo in active {
        let Some(entry) = demo_entry(demo) else {
            continue;
        };
        crate::systems::input::aim_at(scene.input, demo);
        (entry.step)(storage, scene, delta_time);
    }
}
