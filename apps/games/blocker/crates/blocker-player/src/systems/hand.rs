use crate::components::{Motion, Pitch, Yaw};
use crate::data::settings::Settings;
use crate::resources::{Hand, Mining};
use crate::tags::HeldItem;
use crate::theme;
use blocker_blocks::data::items::Item;
use blocker_figure::components::{Eye, Gaze};
use blocker_figure::queries::read_the_hero;
use blocker_figure::resources::Hero;
use blocker_pocket::queries::selected_stack;
use blocker_pocket::resources::Inventory;
use nalgebra_glm::{Vec3, quat_angle_axis, quat_rotate_vec3};
use nightshade::prelude::*;
use nightshade_config::prelude::Config;
use nightshade_graphics::commands::builtins::despawn_entities_with_cache_cleanup;
use nightshade_graphics::commands::builtins::set_mesh_material;
use nightshade_platform::prelude::Time;
use nightshade_render::commands::mesh_state::mesh_state_mark_entity_added;
use nightshade_render::components::material::MaterialRef;
use nightshade_render::components::mesh::RenderMesh;
use nightshade_render::prelude::MeshRenderState;
use nightshade_scene::prelude::{GlobalTransform, LocalTransform, Name, Visibility};

pub fn hold(
    inventory: Res<Inventory>,
    mining: Res<Mining>,
    mut hand: ResMut<Hand>,
    time: Res<Time>,
    hero: Res<Hero>,
    config: Res<Config<Settings>>,
    mut commands: Commands,
    mut bodies: Query<(&Motion, &Yaw, &Pitch, &Eye, &Gaze)>,
) {
    let Some((motion, yaw, pitch, eye, gaze)) =
        read_the_hero(&hero, &mut bodies, |(motion, yaw, pitch, eye, gaze)| {
            (*motion, yaw.0, pitch.0, eye.0, gaze.0)
        })
    else {
        return;
    };
    let carried_overhead = motion == Motion::Glide && config.toggle(Settings::GliderInFirstPerson);
    let wanted = if motion == Motion::Watch || carried_overhead {
        None
    } else {
        selected_stack(&inventory).map(|stack| stack.item)
    };
    if wanted != hand.shown {
        if let Some(entity) = hand.entity.take() {
            commands.queue(move |storage, resources| {
                despawn_entities_with_cache_cleanup(storage, resources, &[entity]);
            });
        }
        hand.shown = wanted;
        if let Some(item) = wanted {
            hand.entity = Some(spawn_hand(&mut commands, item));
        }
    }
    let Some((entity, item)) = hand.entity.zip(hand.shown) else {
        return;
    };

    if mining.block.is_some() {
        hand.swing += time.delta_time.min(0.05);
    }
    if hand.swing > 0.0 {
        hand.swing += time.delta_time;
        if hand.swing > theme::SWING_SECONDS {
            hand.swing = 0.0;
        }
    }
    let swing = (hand.swing / theme::SWING_SECONDS).clamp(0.0, 1.0);
    let arc = (swing * std::f32::consts::PI).sin();

    let rotation = quat_angle_axis(yaw, &Vec3::y()) * quat_angle_axis(pitch, &Vec3::x());
    let forward = gaze;
    let right = quat_rotate_vec3(&rotation, &Vec3::x());
    let up = quat_rotate_vec3(&rotation, &Vec3::y());
    let (offset, tilt, scale) = match item {
        Item::Block(_) => (
            theme::HAND_BLOCK_OFFSET,
            quat_angle_axis(theme::HAND_BLOCK_YAW, &Vec3::y())
                * quat_angle_axis(theme::HAND_BLOCK_PITCH, &Vec3::x()),
            theme::HAND_BLOCK_SCALE,
        ),
        _ => (
            theme::HAND_TOOL_OFFSET,
            quat_angle_axis(theme::HAND_TOOL_YAW, &Vec3::y())
                * quat_angle_axis(theme::HAND_TOOL_PITCH, &Vec3::x()),
            theme::HAND_TOOL_SCALE,
        ),
    };
    let dip = theme::HAND_DIP * arc;
    let place =
        eye + right * (offset.x + dip.x) + up * (offset.y + dip.y) + forward * (offset.z + dip.z);
    let swing_rotation = quat_angle_axis(theme::HAND_SWING_ARC * arc, &Vec3::x());
    commands.set(
        entity,
        LocalTransform {
            translation: place,
            rotation: rotation * swing_rotation * tilt,
            scale: Vec3::repeat(scale),
        },
    );
}

fn spawn_hand(commands: &mut Commands, item: Item) -> Entity {
    let entity = commands.spawn((
        Name(format!("Held {}", item.name())),
        LocalTransform::default(),
        GlobalTransform::default(),
        RenderMesh::new(item.mesh_name()),
        MaterialRef::default(),
        Visibility { visible: true },
    ));
    set_mesh_material(commands, entity, item.material_name());
    commands.add_tag_type::<HeldItem>(entity);
    commands.queue(move |_, resources| {
        mesh_state_mark_entity_added(resources.res_mut::<MeshRenderState>(), entity);
    });
    entity
}
