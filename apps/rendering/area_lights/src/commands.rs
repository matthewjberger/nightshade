use crate::theme;
use nightshade::prelude::*;
use nightshade_graphics::prelude::*;
use nightshade_render::components::material::MaterialRef;
use nightshade_render::contract::{AreaLightShape, Light, LightType};
use nightshade_render::data::material::Material;
use nightshade_render::resources::material_registry::MaterialRegistry;

pub fn dress(commands: &mut Commands<'_>, entity: Entity, material: Material) {
    let name = format!("coat {}", entity.id);
    commands.queue(move |storage, resources| {
        let id = material_registry_insert(
            resources.res_mut::<MaterialRegistry>(),
            name.clone(),
            material.clone(),
        );
        storage.set(entity, MaterialRef::new(id));
    });
}

pub struct Surface {
    pub shape: AreaLightShape,
    pub at: Vec3,
    pub rotation: Quat,
    pub width: f32,
    pub height: f32,
    pub radius: f32,
}

pub fn light_surface(
    commands: &mut Commands<'_>,
    name: String,
    surface: Surface,
    tint: Vec3,
    intensity: f32,
    range: f32,
) {
    commands.spawn((
        Name(name),
        LocalTransform {
            translation: surface.at,
            rotation: surface.rotation,
            scale: Vec3::new(1.0, 1.0, 1.0),
        },
        GlobalTransform::default(),
        Light {
            light_type: LightType::Area,
            color: tint,
            intensity,
            range,
            area_shape: surface.shape,
            area_width: surface.width,
            area_height: surface.height,
            area_radius: surface.radius,
            cast_shadows: false,
            ..Default::default()
        },
    ));

    let (mesh, scale) = match surface.shape {
        AreaLightShape::Rectangle => (
            "Cube",
            Vec3::new(surface.width, surface.height, theme::SURFACE_DEPTH),
        ),
        AreaLightShape::Disk => (
            "Cylinder",
            Vec3::new(
                surface.radius * 2.0,
                theme::SURFACE_DEPTH,
                surface.radius * 2.0,
            ),
        ),
        AreaLightShape::Sphere => (
            "Sphere",
            Vec3::new(surface.radius, surface.radius, surface.radius),
        ),
        AreaLightShape::Tube => (
            "Cylinder",
            Vec3::new(surface.radius * 2.0, surface.width, surface.radius * 2.0),
        ),
    };
    let marker = spawn_mesh_at(commands, mesh, surface.at, scale);
    commands.set(
        marker,
        LocalTransform {
            translation: surface.at,
            rotation: surface.rotation * marker_turn(surface.shape),
            scale,
        },
    );
    dress(
        commands,
        marker,
        Material {
            base_color: [0.0, 0.0, 0.0, 1.0],
            emissive_factor: [
                tint.x * theme::MARKER_GLOW,
                tint.y * theme::MARKER_GLOW,
                tint.z * theme::MARKER_GLOW,
            ],
            unlit: true,
            ..Default::default()
        },
    );
}

fn marker_turn(shape: AreaLightShape) -> Quat {
    match shape {
        AreaLightShape::Rectangle | AreaLightShape::Sphere => Quat::identity(),
        AreaLightShape::Disk => {
            nalgebra_glm::quat_angle_axis(std::f32::consts::FRAC_PI_2, &Vec3::x())
        }
        AreaLightShape::Tube => {
            nalgebra_glm::quat_angle_axis(-std::f32::consts::FRAC_PI_2, &Vec3::z())
        }
    }
}
