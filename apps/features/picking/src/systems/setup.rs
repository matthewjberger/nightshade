use crate::components::{DebugVolume, DemoMesh};
use crate::resources::PickingResources;
use crate::systems::geometry::debug_volume_wireframe;
use crate::systems::panel;
use crate::theme::{CAMERA_DISTANCE, CAMERA_PITCH, CAMERA_TARGET, CAMERA_YAW, MESH_LAYOUT};
use nightshade::prelude::*;
use nightshade_camera::prelude::*;
use nightshade_camera_controller::commands::spawn_pan_orbit_camera_at;
use nightshade_graphics::prelude::*;
use nightshade_render::components::lines::Lines;

pub fn initialize(
    mut picking: ResMut<PickingResources>,
    mut debug_draw: ResMut<DebugDraw>,
    mut settings: ResMut<RenderSettings>,
    mut registry: ResMut<nightshade_render::resources::material_registry::MaterialRegistry>,
    mut active_camera: ResMut<ActiveCamera>,
    mut commands: Commands,
    mut build: nightshade_ui::commands::builder::UiBuild,
) {
    let picking = &mut *picking;
    debug_draw.show_grid = true;
    settings.atmosphere = Atmosphere::Sky;
    picking.show_debug_volumes = true;
    picking.spin_speed = 1.0;

    spawn_sun(&mut commands);

    for (mesh_type, position, color) in MESH_LAYOUT {
        let entity = spawn_mesh(&mut commands, mesh_type, position, Vec3::new(1.0, 1.0, 1.0));
        set_material(
            &mut registry,
            &mut commands,
            entity,
            format!("{}_{}", mesh_type, entity.id),
            Material {
                base_color: color.into(),
                ..Default::default()
            },
        );
        commands.set(entity, DemoMesh { label: mesh_type });

        let radius =
            nightshade_render::data::bounding_volume::BoundingVolume::from_mesh_type(mesh_type)
                .sphere_radius;
        spawn_debug_volume(&mut commands, mesh_type, entity, position, radius);
    }

    let camera = spawn_pan_orbit_camera_at(
        &mut commands,
        CAMERA_TARGET,
        CAMERA_DISTANCE,
        CAMERA_YAW,
        CAMERA_PITCH,
        "Picking Demo Camera".to_string(),
    );
    active_camera.0 = Some(camera);

    picking.preview_lines_entity = Some(spawn_preview_marker(&mut commands));
    picking.handles = panel::build(
        build.context(),
        picking.mode,
        picking.show_debug_volumes,
        picking.spin_speed,
    );
}

fn spawn_preview_marker(commands: &mut Commands) -> Entity {
    commands.spawn((
        Lines::default(),
        Visibility { visible: true },
        GlobalTransform::default(),
    ))
}

fn spawn_debug_volume(
    commands: &mut Commands,
    name: &str,
    mesh: Entity,
    position: Vec3,
    radius: f32,
) -> Entity {
    commands.spawn((
        Name(format!("{name} Debug Volume")),
        LocalTransform {
            translation: position,
            scale: Vec3::new(1.0, 1.0, 1.0),
            ..Default::default()
        },
        GlobalTransform::default(),
        Lines::new(debug_volume_wireframe(radius)),
        Visibility { visible: true },
        DebugVolume { mesh },
    ))
}
