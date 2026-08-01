use nightshade::ecs::Entity;
use nightshade::prelude::*;
use nightshade_render::components::render_world;
use nightshade_render::contract::Light;
use nightshade_render::data::config::Atmosphere;
use nightshade_render::resources::light_state::LightSyncState;

pub fn render_sync_light_system(
    mut state: ResMut<LightSyncState>,
    mesh_state: Res<nightshade_render::resources::mesh_state::MeshRenderState>,
    mut commands: Commands,
    changed: Query<(&Light, &nightshade_scene::contract::GlobalTransform), Changed<Light>>,
    mut lights: Query<(&Light, &nightshade_scene::contract::GlobalTransform)>,
    mut lit: Query<(&nightshade_render::components::light::RenderLightData,)>,
) {
    let mut count = 0usize;
    lights.for_each_mut(|_entity, _| count += 1);
    let count_changed = count != state.entity_count;
    state.entity_count = count;

    let mut touched: Vec<(
        Entity,
        nightshade_render::components::light::RenderLightData,
        nalgebra_glm::Mat4,
    )> = Vec::new();
    changed.for_each(|entity, (light, transform)| {
        touched.push((entity, render_light(light), transform.0));
    });
    for entity in mesh_state.transform_dirty.iter().copied() {
        lights.for_entity(entity, |(light, transform)| {
            touched.push((entity, render_light(light), transform.0));
        });
    }
    if count_changed {
        let mut lit_entities: std::collections::HashSet<Entity> = std::collections::HashSet::new();
        lit.for_each_mut(|entity, _| {
            lit_entities.insert(entity);
        });
        let mut rows: Vec<(
            Entity,
            nightshade_render::components::light::RenderLightData,
            nalgebra_glm::Mat4,
        )> = Vec::new();
        lights.for_each_mut(|entity, (light, transform)| {
            rows.push((entity, render_light(light), transform.0));
        });
        let alive: std::collections::HashSet<Entity> =
            rows.iter().map(|(entity, _, _)| *entity).collect();
        for entity in &lit_entities {
            if !alive.contains(entity) {
                commands
                    .remove::<(nightshade_render::components::light::RenderLightData,)>(*entity);
            }
        }
        for row in rows {
            if !lit_entities.contains(&row.0) {
                touched.push(row);
            }
        }
    }
    touched.sort_by_key(|(entity, _, _)| (entity.id, entity.generation));
    touched.dedup_by_key(|(entity, _, _)| (entity.id, entity.generation));
    for (entity, light, transform) in touched {
        commands.set(entity, light);
        commands.set(entity, render_world::Transform(transform));
    }
}

fn render_light(light: &Light) -> nightshade_render::components::light::RenderLightData {
    nightshade_render::components::light::RenderLightData {
        light_type: match light.light_type {
            nightshade_render::contract::LightType::Directional => {
                nightshade_render::data::config::RenderLightType::Directional
            }
            nightshade_render::contract::LightType::Point => {
                nightshade_render::data::config::RenderLightType::Point
            }
            nightshade_render::contract::LightType::Spot => {
                nightshade_render::data::config::RenderLightType::Spot
            }
            nightshade_render::contract::LightType::Area => {
                nightshade_render::data::config::RenderLightType::Area
            }
        },
        color: light.color,
        intensity: light.intensity,
        range: light.range,
        inner_cone_angle: light.inner_cone_angle,
        outer_cone_angle: light.outer_cone_angle,
        cast_shadows: light.cast_shadows,
        shadow_bias: light.shadow_bias,
        shadow_resolution: light.shadow_resolution,
        shadow_distance: light.shadow_distance,
        cookie_texture: light.cookie_texture.clone(),
        area_shape: match light.area_shape {
            nightshade_render::contract::AreaLightShape::Rectangle => {
                nightshade_render::data::config::RenderAreaLightShape::Rectangle
            }
            nightshade_render::contract::AreaLightShape::Disk => {
                nightshade_render::data::config::RenderAreaLightShape::Disk
            }
            nightshade_render::contract::AreaLightShape::Sphere => {
                nightshade_render::data::config::RenderAreaLightShape::Sphere
            }
            nightshade_render::contract::AreaLightShape::Tube => {
                nightshade_render::data::config::RenderAreaLightShape::Tube
            }
        },
        area_width: light.area_width,
        area_height: light.area_height,
        area_radius: light.area_radius,
        area_two_sided: light.area_two_sided,
        area_emissive_texture: light.area_emissive_texture.clone(),
        shadow_normal_bias: light.shadow_normal_bias,
        shadow_softness: light.shadow_softness,
    }
}

pub fn day_night_cycle_system(
    mut renderer_state: ResMut<nightshade_render::data::config::RendererState>,
    render_settings: Res<nightshade_render::data::config::RenderSettings>,
    time: Res<nightshade_platform::contract::time::Time>,
    mut sun_row: Query<(
        &mut nightshade_render::contract::Light,
        &mut nightshade_scene::contract::LocalTransform,
    )>,
) {
    if !renderer_state.day_night.auto_cycle {
        return;
    }
    if !matches!(render_settings.atmosphere, Atmosphere::DayNight) {
        return;
    }

    let speed = renderer_state.day_night.speed;
    renderer_state.day_night.hour += speed * time.delta_time;
    if renderer_state.day_night.hour >= 24.0 {
        renderer_state.day_night.hour -= 24.0;
    }

    let Some(sun) = renderer_state.day_night.sun_entity else {
        return;
    };

    let hour = renderer_state.day_night.hour;
    let direction = sun_direction(hour);
    let elevation = direction.y.clamp(0.0, 1.0);
    let intensity = 3.5 * elevation.sqrt();

    let warm = nalgebra_glm::vec3(1.0, 0.7, 0.4);
    let white = nalgebra_glm::vec3(1.0, 0.95, 0.8);
    let height = (elevation / 0.35).clamp(0.0, 1.0);
    let color = nalgebra_glm::lerp(&warm, &white, height * height * (3.0 - 2.0 * height));

    sun_row.for_entity(sun, |(light, transform)| {
        light.intensity = intensity;
        light.color = color;

        transform.translation = direction * 100.0;
        let look_direction = -direction;
        let up = nalgebra_glm::Vec3::y();
        let right = nalgebra_glm::normalize(&nalgebra_glm::cross(&look_direction, &up));
        if right.norm() > 0.001 {
            let corrected_up = nalgebra_glm::cross(&right, &look_direction);
            transform.rotation = nalgebra_glm::mat3_to_quat(&nalgebra_glm::Mat3::from_columns(&[
                right,
                corrected_up,
                -look_direction,
            ]));
        }
    });
}

fn sun_direction(hour: f32) -> nalgebra_glm::Vec3 {
    let sun_angle = (hour - 6.0) / 12.0 * std::f32::consts::PI;
    nalgebra_glm::normalize(&nalgebra_glm::vec3(-sun_angle.cos(), sun_angle.sin(), -0.3))
}
