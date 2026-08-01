use crate::data::Panel;
use crate::resources::Shown;
use crate::theme;
use nightshade::ecs::Storage;
use nightshade::ecs::system_param::{Commands, Res, ResMut};
use nightshade_config::contract::Config;
use nightshade_platform::contract::time::Time;
use nightshade_readout::Readout;
use nightshade_render::commands::material_registry::set_material;
use nightshade_render::data::material::Material;
use nightshade_render::resources::material_registry::MaterialRegistry;
use nightshade_scene::contract::LocalTransform;

pub fn follow_the_settings(
    time: Res<Time>,
    mut shown: ResMut<Shown>,
    mut material_registry: ResMut<MaterialRegistry>,
    mut readout: ResMut<Readout>,
    config: Res<Config<Panel>>,
    mut commands: Commands,
    storage: &mut Storage,
) {
    let delta = time.delta_time;
    let spin = config.number(Panel::Spin);
    let lift = config.number(Panel::Lift);
    let palette = config.choice(Panel::Palette);
    let turning = config.toggle(Panel::Turning);

    let turn = {
        if turning {
            shown.turn += delta * spin;
        }
        shown.turn
    };

    let bodies: Vec<_> = storage
        .query_ref::<(&LocalTransform,)>()
        .iter()
        .filter(|(_entity, (transform,))| transform.translation.y > 1.0)
        .map(|(entity, _)| entity)
        .collect();
    for body in &bodies {
        if let Some(transform) = storage.get_mut::<LocalTransform>(*body) {
            transform.rotation = nalgebra_glm::quat_angle_axis(turn, &nalgebra_glm::Vec3::y());
            transform.translation.y = 1.2 + lift;
        }
    }

    let wanted = theme::TINTS[palette.min(theme::TINTS.len() - 1)];
    if shown.tint != wanted {
        shown.tint = wanted;
        {
            let commands = &mut commands;

            let materials = &mut *material_registry;
            let bodies: Vec<_> = storage
                .query_ref::<(&LocalTransform,)>()
                .iter()
                .filter(|(_entity, (transform,))| transform.translation.y > 1.0)
                .map(|(entity, _)| entity)
                .collect();
            for body in bodies {
                set_material(
                    materials,
                    commands,
                    body,
                    format!("{}_{}", theme::TITLE, body.id),
                    Material {
                        base_color: wanted,
                        roughness: 0.8,
                        metallic: 0.0,
                        ..Default::default()
                    },
                );
            }
        };
    }

    readout.notes.clear();
    readout
        .notes
        .push(format!("spin {spin:.2}, lift {lift:.2}"));
    readout.notes.push(format!(
        "palette {palette}, {}",
        if turning { "turning" } else { "still" }
    ));
}
