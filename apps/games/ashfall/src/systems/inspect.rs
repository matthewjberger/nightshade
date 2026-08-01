use crate::commands::{post_message, stow, take_out};
use crate::components::ItemKind;
use crate::data::ask::Ask;
use crate::data::rules::*;
use crate::data::theme;
use crate::queries::{
    item_look, item_name, item_story, moved_across, moved_down, spin_across, spin_down,
};
use crate::resources::{AshfallResources, Screen};
use crate::systems::textures::solid;
use crate::tags::InspectPiece;
use nightshade::prelude::*;
use nightshade_camera::prelude::*;
use nightshade_graphics::commands::builtins::spawn_mesh_at;
use nightshade_input::Actions;
use nightshade_platform::prelude::*;
use nightshade_scene::prelude::*;
use nightshade_ui::prelude::ui_set_text;

pub fn build(mut resources: ResMut<AshfallResources>, mut commands: Commands) {
    let camera = commands.spawn((
        Name("Inspect Camera".to_string()),
        LocalTransform {
            translation: theme::INSPECT_ORIGIN + Vec3::new(0.0, 0.0, theme::INSPECT_BACK),
            ..Default::default()
        },
        GlobalTransform::default(),
        Camera {
            projection: Projection::Perspective(PerspectiveCamera {
                aspect_ratio: None,
                y_fov_rad: 40.0_f32.to_radians(),
                z_far: None,
                z_near: 0.01,
            }),
            smoothing: None,
        },
    ));
    commands.spawn((
        Name("Inspect Light".to_string()),
        LocalTransform {
            translation: theme::INSPECT_ORIGIN + Vec3::new(1.4, 1.8, 2.2),
            ..Default::default()
        },
        GlobalTransform::default(),
        nightshade_graphics::prelude::Light {
            light_type: nightshade_graphics::prelude::LightType::Point,
            color: theme::INSPECT_LIGHT,
            intensity: theme::INSPECT_INTENSITY,
            range: 12.0,
            cast_shadows: false,
            ..Default::default()
        },
    ));
    resources.inspect.camera = Some(camera);
}

pub fn update(
    time: Res<Time>,
    input: Res<Input>,
    actions: Res<Actions<Ask>>,
    mut resources: ResMut<AshfallResources>,
    mut commands: Commands,
    mut pieces: Query<(&mut LocalTransform,), WithTag<InspectPiece>>,
) {
    let resources = &mut *resources;
    let open = resources.screen == Screen::Inspect;
    if open != resources.inspect.showing {
        resources.inspect.showing = open;
        if open {
            enter(resources, &mut commands);
        } else {
            leave(resources, &mut commands);
        }
    }
    if !open {
        return;
    }

    if resources.inspect.kind == Some(ItemKind::Case) {
        turn_the_wheels(resources, &actions, &mut commands);
        return;
    }

    let delta = time.delta_time;
    let across = spin_across(&input, &actions) * SPIN_RATE * delta;
    let down = spin_down(&input, &actions) * SPIN_RATE * delta;
    if across.abs() > 0.0001 || down.abs() > 0.0001 {
        let yaw = nalgebra_glm::quat_angle_axis(across, &Vec3::y());
        let pitch = nalgebra_glm::quat_angle_axis(down, &Vec3::x());
        resources.inspect.spin = nalgebra_glm::quat_normalize(&nalgebra_glm::quat_cross(
            &nalgebra_glm::quat_cross(&yaw, &pitch),
            &resources.inspect.spin,
        ));
    }
    let spin = resources.inspect.spin;
    pieces.for_each_mut(|_entity, (transform,)| transform.rotation = spin);

    let underside = nalgebra_glm::quat_rotate_vec3(&spin, &Vec3::new(0.0, -1.0, 0.0));
    let facing = underside.z > CATCH_FACING;
    let holds_a_catch = resources.inspect.kind == Some(ItemKind::Lockbox);
    resources.inspect.found_catch = facing && holds_a_catch;

    if resources.inspect.found_catch && actions.just_pressed(Ask::Confirm) {
        open_the_box(resources);
        return;
    }
    if actions.just_pressed(Ask::Pause) {
        resources.pending_screen = Some(Screen::Inventory);
    }
    write(resources, &mut commands);
}

fn turn_the_wheels(
    resources: &mut AshfallResources,
    actions: &Actions<Ask>,
    commands: &mut Commands,
) {
    let across = moved_across(actions);
    if across != 0 {
        let wheels = CASE_DIGITS as i32;
        let wheel = resources.inspect.wheel as i32 + across;
        resources.inspect.wheel = wheel.rem_euclid(wheels) as usize;
    }
    let down = moved_down(actions);
    if down != 0 {
        let wheel = resources.inspect.wheel;
        let digit = resources.inspect.wheels[wheel] as i32 - down;
        resources.inspect.wheels[wheel] = digit.rem_euclid(10) as u32;
    }
    resources.inspect.found_catch = resources.inspect.wheels == CASE_CODE;
    if resources.inspect.found_catch && actions.just_pressed(Ask::Confirm) {
        open_the_case(resources);
        return;
    }
    if actions.just_pressed(Ask::Pause) {
        resources.pending_screen = Some(Screen::Inventory);
    }
    write(resources, commands);
}

fn open_the_case(resources: &mut AshfallResources) {
    take_out(&mut resources.satchel, resources.inspect.slot);
    if stow(&mut resources.satchel, ItemKind::BrassKey) {
        post_message(
            &mut resources.run,
            "The wheels line up and the lid comes off in your hand. A brass key.",
        );
    } else {
        stow(&mut resources.satchel, ItemKind::Case);
        post_message(
            &mut resources.run,
            "Your hands are too full to take it out.",
        );
    }
    resources.pending_screen = Some(Screen::Inventory);
}

fn enter(resources: &mut AshfallResources, commands: &mut Commands) {
    let Some(kind) = resources.inspect.kind else {
        return;
    };
    resources.inspect.spin = Quat::identity();
    resources.inspect.wheels = [0; CASE_DIGITS];
    resources.inspect.wheel = 0;
    let (mesh, scale, color, glow) = item_look(kind);
    let entity = spawn_mesh_at(
        commands,
        mesh,
        theme::INSPECT_ORIGIN,
        scale * theme::INSPECT_SCALE,
    );
    crate::commands::attach_material(
        commands,
        entity,
        format!("ashfall_inspect_{kind:?}"),
        solid(color, glow),
    );
    commands.add_tag_type::<InspectPiece>(entity);
    resources.inspect.item = Some(entity);
    if let Some(camera) = resources.inspect.camera {
        commands.queue(move |_, resources| {
            resources.res_mut::<ActiveCamera>().0 = Some(camera);
        });
    }
}

fn leave(resources: &mut AshfallResources, commands: &mut Commands) {
    if let Some(item) = resources.inspect.item.take() {
        commands.despawn_recursive(item);
    }
    if let Some(camera) = resources.player.eyes {
        commands.queue(move |_, resources| {
            resources.res_mut::<ActiveCamera>().0 = Some(camera);
        });
    }
}

fn open_the_box(resources: &mut AshfallResources) {
    let slot = resources.inspect.slot;
    take_out(&mut resources.satchel, slot);
    if stow(&mut resources.satchel, ItemKind::WardKey) {
        post_message(
            &mut resources.run,
            "The base slides back. A ward key, and a tag reading eleven.",
        );
    } else {
        post_message(
            &mut resources.run,
            "The base slides back, and you have nowhere to put what is inside.",
        );
        stow(&mut resources.satchel, ItemKind::Lockbox);
    }
    resources.pending_screen = Some(Screen::Inventory);
}

fn write(resources: &AshfallResources, commands: &mut Commands) {
    let handles = resources.ui;
    let Some(kind) = resources.inspect.kind else {
        return;
    };
    if let Some(node) = handles.inspect_name {
        ui_set_text(commands, node, &item_name(kind));
    }
    if let Some(node) = handles.inspect_body {
        ui_set_text(commands, node, item_story(kind));
    }
    if let Some(node) = handles.inspect_hint {
        let hint = if kind == ItemKind::Case {
            let wheels: Vec<String> = resources
                .inspect
                .wheels
                .iter()
                .enumerate()
                .map(|(index, digit)| {
                    if index == resources.inspect.wheel {
                        format!("[{digit}]")
                    } else {
                        format!(" {digit} ")
                    }
                })
                .collect();
            if resources.inspect.found_catch {
                format!("{}   the lid gives", wheels.join(""))
            } else {
                wheels.join("")
            }
        } else if resources.inspect.found_catch {
            "A catch, sunk into the base.".to_string()
        } else {
            "Turn it over.".to_string()
        };
        ui_set_text(commands, node, &hint);
    }
}
