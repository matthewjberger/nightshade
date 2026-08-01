use crate::data::Item;
use crate::queries::selected_stack;
use crate::resources::{Hand, Inventory, Player};
use crate::tags::HeldItem;
use crate::theme;
use nalgebra_glm::{Vec3, quat_angle_axis, quat_rotate_vec3};
use nightshade::prelude::*;
use nightshade_graphics::commands::builtins::despawn_entities_with_cache_cleanup;
use nightshade_graphics::commands::builtins::set_mesh_material;
use nightshade_platform::prelude::Time;
use nightshade_render::commands::mesh_state::mesh_state_mark_entity_added;
use nightshade_render::components::material::MaterialRef;
use nightshade_render::components::mesh::RenderMesh;
use nightshade_render::prelude::MeshRenderState;
use nightshade_scene::prelude::{GlobalTransform, LocalTransform, Name, Visibility};

pub fn hold(
    player: Res<Player>,
    inventory: Res<Inventory>,
    mut hand: ResMut<Hand>,
    time: Res<Time>,
    mut commands: Commands,
) {
    let wanted = if player.watching {
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

    if hand.swing > 0.0 {
        hand.swing += time.delta_time;
        if hand.swing > theme::SWING_SECONDS {
            hand.swing = 0.0;
        }
    }
    let swing = (hand.swing / theme::SWING_SECONDS).clamp(0.0, 1.0);
    let arc = (swing * std::f32::consts::PI).sin();

    let rotation =
        quat_angle_axis(player.yaw, &Vec3::y()) * quat_angle_axis(player.pitch, &Vec3::x());
    let forward = player.forward;
    let right = quat_rotate_vec3(&rotation, &Vec3::x());
    let up = quat_rotate_vec3(&rotation, &Vec3::y());
    let (offset, tilt, scale) = match item {
        Item::Block(_) => (
            Vec3::new(0.34, -0.30, 0.62),
            quat_angle_axis(-0.6, &Vec3::y()) * quat_angle_axis(0.08, &Vec3::x()),
            0.24,
        ),
        _ => (
            Vec3::new(0.36, -0.26, 0.6),
            quat_angle_axis(std::f32::consts::PI - 0.5, &Vec3::y())
                * quat_angle_axis(-0.35, &Vec3::x()),
            0.5,
        ),
    };
    let dip = Vec3::new(0.0, -0.12, 0.08) * arc;
    let place = player.eye
        + right * (offset.x + dip.x)
        + up * (offset.y + dip.y)
        + forward * (offset.z + dip.z);
    let swing_rotation = quat_angle_axis(-1.1 * arc, &Vec3::x());
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
