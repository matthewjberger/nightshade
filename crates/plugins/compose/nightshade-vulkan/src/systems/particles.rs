use crate::pre_render::Renderer;
use crate::renderer::particles::{
    GpuEmitter, MAX_EMITTERS, MAX_SPAWN_PER_FRAME, PARTICLES_PASS, ParticleFrameData,
    PendingTexture, create_pass, hand_over,
};
use crate::renderer::{VulkanRenderer, install_scene_pass, pass_setup};
use nightshade::prelude::*;
use nightshade_particles::contract::data::{
    emitter_shape_code, emitter_type_code, pack_color_gradient, pack_value_curve,
};
use nightshade_particles::contract::{ParticleEmitter, ParticleFrame, ParticleTextures};
use nightshade_render::queries::projection::matrix_columns;

pub fn install(renderer: &mut VulkanRenderer) {
    let setup = pass_setup(renderer);
    match create_pass(&setup) {
        Ok(pass) => install_scene_pass(renderer, Box::new(pass)),
        Err(error) => tracing::error!("the particles pass was not installed: {error}"),
    }
}

pub fn upload_particles(
    mut renderer: Renderer,
    frame: Option<Res<ParticleFrame>>,
    textures: Option<ResMut<ParticleTextures>>,
    mut offsets: ResMut<crate::resources::SpawnOffsets>,
    time: Res<nightshade_platform::contract::time::Time>,
) {
    let (Some(frame), Some(mut textures)) = (frame, textures) else {
        return;
    };
    let Some(view) = renderer.view.clone() else {
        return;
    };
    let right = [
        view.camera_right.x,
        view.camera_right.y,
        view.camera_right.z,
        0.0,
    ];
    let up = [view.camera_up.x, view.camera_up.y, view.camera_up.z, 0.0];

    let mut emitters = Vec::new();
    for (entity, emitter) in &frame.emitters {
        if emitters.len() >= MAX_EMITTERS as usize {
            break;
        }
        if !emitter.enabled {
            offsets.offsets.insert(*entity, emitter.accumulated_spawn);
            continue;
        }
        let offset = offsets.offsets.get(entity).copied().unwrap_or(0.0);
        let spawn_count = (emitter.accumulated_spawn - offset).max(0.0) as u32;
        if spawn_count == 0 {
            continue;
        }
        let taken = spawn_count.min(MAX_SPAWN_PER_FRAME);
        offsets.offsets.insert(*entity, offset + taken as f32);
        emitters.push(pack(emitter, emitters.len() as u32, taken));
    }

    let pending = std::mem::take(&mut textures.pending)
        .into_iter()
        .map(|upload| PendingTexture {
            slot: upload.slot,
            rgba: upload.rgba_data,
            width: upload.width,
            height: upload.height,
        })
        .collect();

    let data = ParticleFrameData {
        emitters,
        delta_time: time.delta_time,
        time: time.uptime_seconds as f32,
        view_projection: matrix_columns(&view.view_projection),
        camera_right: right,
        camera_up: up,
        textures: pending,
    };

    let Some(pass) = renderer
        .passes
        .iter_mut()
        .find(|(_, pass)| pass.name() == PARTICLES_PASS)
        .map(|(_, pass)| pass.as_mut())
    else {
        return;
    };
    if let Some(pass) = (pass as &mut dyn std::any::Any).downcast_mut() {
        hand_over(pass, data);
    }
}

fn pack(emitter: &ParticleEmitter, index: u32, spawn_count: u32) -> GpuEmitter {
    let (shape_type, shape_params) = emitter_shape_code(&emitter.shape);
    let emitter_type = emitter_type_code(emitter.emitter_type);
    let (gradient, stops) = pack_color_gradient(&emitter.color_gradient);
    let (size_curve, size_curve_count) = pack_value_curve(&emitter.size_curve);
    let (opacity_curve, opacity_curve_count) = pack_value_curve(&emitter.opacity_curve);

    GpuEmitter {
        position: [
            emitter.position.x,
            emitter.position.y,
            emitter.position.z,
            0.0,
        ],
        direction: [
            emitter.direction.x,
            emitter.direction.y,
            emitter.direction.z,
            0.0,
        ],
        velocity_range: [
            emitter.initial_velocity_min,
            emitter.initial_velocity_max,
            emitter.velocity_spread,
            0.0,
        ],
        lifetime_range: [
            emitter.particle_lifetime_min,
            emitter.particle_lifetime_max,
            0.0,
            0.0,
        ],
        size_range: [emitter.size_start, emitter.size_end, 0.0, 0.0],
        gravity: [emitter.gravity.x, emitter.gravity.y, emitter.gravity.z, 0.0],
        color_gradient: gradient,
        gradient_count: stops,
        spawn_count,
        emitter_id: index,
        shape_type,
        shape_params,
        turbulence: [
            emitter.turbulence_strength,
            emitter.turbulence_frequency,
            0.0,
            0.0,
        ],
        emissive_strength: emitter.emissive_strength,
        drag: emitter.drag,
        emitter_type,
        texture_index: emitter.texture_index,
        size_curve,
        size_curve_count,
        opacity_curve,
        opacity_curve_count,
        ..GpuEmitter::default()
    }
}
