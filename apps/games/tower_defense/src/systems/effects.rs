use crate::commands::Scene;
use crate::components::{Age, Lifetime};
use crate::data::EffectType;
use crate::queries::read_or_default;
use crate::resources::TowerDefenseResources;
use crate::systems::despawn_entity;
use nightshade::prelude::*;
use nightshade_graphics::prelude::*;
use nightshade_particles::contract::ParticleEmitter;
use nightshade_render::commands::material_registry::material_registry_mutate_id;
use rand::Rng;

pub fn create_explosion_effect(
    game_world: &mut TowerDefenseResources,
    scene: &mut Scene<'_, '_>,
    position: Vec3,
) {
    for _ in 0..8 {
        let offset = nalgebra_glm::vec3(
            rand::rng().random_range(-0.5..0.5),
            rand::rng().random_range(0.0..0.5),
            rand::rng().random_range(-0.5..0.5),
        );

        let particle = spawn_mesh(
            scene.commands,
            "Sphere",
            position + offset,
            nalgebra_glm::vec3(0.15, 0.15, 0.15),
        );

        set_material(
            scene.registry,
            scene.commands,
            particle,
            format!("ExplosionParticle_{}", particle.id),
            Material {
                base_color: [1.0, 0.5, 0.0, 0.8],
                emissive_factor: [1.0, 0.3, 0.0],
                alpha_mode: AlphaMode::Blend,
                ..Default::default()
            },
        );

        scene.commands.set(particle, EffectType::Explosion);
        scene.commands.set(particle, Lifetime(0.5));
        scene.commands.set(particle, Age(0.0));
        game_world.effects_list.push(particle);
    }
}

pub fn create_death_effect(scene: &mut Scene<'_, '_>, position: Vec3) {
    let entity = scene.commands.spawn(());
    let mut smoke_emitter = ParticleEmitter::smoke(position);
    smoke_emitter.one_shot = true;
    smoke_emitter.burst_count = 30;
    smoke_emitter.spawn_rate = 0.0;
    smoke_emitter.particle_lifetime_min = 0.8;
    smoke_emitter.particle_lifetime_max = 1.5;
    smoke_emitter.initial_velocity_min = 1.5;
    smoke_emitter.initial_velocity_max = 3.0;
    smoke_emitter.size_start = 0.3;
    smoke_emitter.size_end = 0.8;
    smoke_emitter.velocity_spread = 0.6;
    smoke_emitter.shape = nightshade_particles::contract::EmitterShape::Sphere { radius: 0.2 };
    scene.commands.set(entity, smoke_emitter);
}

pub fn create_poison_bubble_effect(
    game_world: &mut TowerDefenseResources,
    scene: &mut Scene<'_, '_>,
    position: Vec3,
) {
    let offset = nalgebra_glm::vec3(
        rand::rng().random_range(-0.2..0.2),
        rand::rng().random_range(0.0..0.5),
        rand::rng().random_range(-0.2..0.2),
    );

    let particle = spawn_mesh(
        scene.commands,
        "Sphere",
        position + offset,
        nalgebra_glm::vec3(0.08, 0.08, 0.08),
    );

    set_material(
        scene.registry,
        scene.commands,
        particle,
        format!("PoisonBubble_{}", particle.id),
        Material {
            base_color: [0.5, 0.0, 0.8, 0.6],
            alpha_mode: AlphaMode::Blend,
            ..Default::default()
        },
    );

    scene.commands.set(particle, EffectType::PoisonBubble);
    scene.commands.set(particle, Lifetime(2.0));
    scene.commands.set(particle, Age(0.0));
    game_world.effects_list.push(particle);
}

pub fn create_muzzle_flash(
    game_world: &mut TowerDefenseResources,
    scene: &mut Scene<'_, '_>,
    position: Vec3,
) {
    let flash = spawn_mesh(
        scene.commands,
        "Sphere",
        position,
        nalgebra_glm::vec3(0.6, 0.6, 0.6),
    );

    set_material(
        scene.registry,
        scene.commands,
        flash,
        format!("MuzzleFlash_{}", flash.id),
        Material {
            base_color: [1.0, 1.0, 0.0, 1.0],
            emissive_factor: [2.0, 2.0, 0.0],
            ..Default::default()
        },
    );

    scene.commands.set(flash, EffectType::Explosion);
    scene.commands.set(flash, Lifetime(0.15));
    scene.commands.set(flash, Age(0.0));
    game_world.effects_list.push(flash);

    for index in 0..8 {
        let angle = (index as f32 / 8.0) * std::f32::consts::TAU;
        let offset = nalgebra_glm::vec3(angle.cos() * 0.3, 0.0, angle.sin() * 0.3);

        let smoke = spawn_mesh(
            scene.commands,
            "Sphere",
            position + offset,
            nalgebra_glm::vec3(0.4, 0.4, 0.4),
        );

        set_material(
            scene.registry,
            scene.commands,
            smoke,
            format!("MuzzleSmoke_{}", smoke.id),
            Material {
                base_color: [0.3, 0.3, 0.3, 0.8],
                alpha_mode: AlphaMode::Blend,
                ..Default::default()
            },
        );

        scene.commands.set(smoke, EffectType::Explosion);
        scene.commands.set(smoke, Lifetime(2.0));
        scene.commands.set(smoke, Age(0.0));
        game_world.effects_list.push(smoke);
    }
}

pub fn update_visual_effects(
    game_world: &mut TowerDefenseResources,
    scene: &mut Scene<'_, '_>,
    delta_time: f32,
) {
    let entities: Vec<Entity> = scene.storage.query_entities::<(EffectType,)>().collect();
    let mut effects_to_remove = Vec::new();

    for entity in entities {
        if let Some(effect_type) = scene.storage.get::<EffectType>(entity).copied() {
            let lifetime = read_or_default::<Lifetime>(scene, entity).0;
            let age = read_or_default::<Age>(scene, entity).0 + delta_time;

            if age >= lifetime {
                effects_to_remove.push(entity);
                continue;
            }

            scene.commands.set(entity, Age(age));

            match effect_type {
                EffectType::Explosion => {
                    let progress = age / lifetime;
                    let scale_factor = 1.0 + progress * 2.0;

                    if let Some(transform) = scene
                        .storage
                        .get_mut::<nightshade_scene::contract::LocalTransform>(entity)
                    {
                        transform.scale = nalgebra_glm::vec3(
                            0.15 * scale_factor,
                            0.15 * scale_factor,
                            0.15 * scale_factor,
                        );
                    }

                    if let Some(material_ref) = scene
                        .storage
                        .get::<nightshade_render::components::material::MaterialRef>(entity)
                        .cloned()
                    {
                        material_registry_mutate_id(scene.registry, material_ref.id, |material| {
                            let alpha = 1.0 - progress;
                            material.base_color[3] = alpha * 0.8;
                        });
                    }
                }
                EffectType::PoisonBubble => {
                    if let Some(transform) = scene
                        .storage
                        .get_mut::<nightshade_scene::contract::LocalTransform>(entity)
                    {
                        transform.translation.y += delta_time * 0.5;
                    }

                    if let Some(material_ref) = scene
                        .storage
                        .get::<nightshade_render::components::material::MaterialRef>(entity)
                        .cloned()
                    {
                        material_registry_mutate_id(scene.registry, material_ref.id, |material| {
                            let progress = age / lifetime;
                            let alpha = 1.0 - progress;
                            material.base_color[3] = alpha * 0.6;
                        });
                    }
                }
            }
        }
    }

    for entity in effects_to_remove {
        if let Some(index) = game_world
            .effects_list
            .iter()
            .position(|&candidate| candidate == entity)
        {
            game_world.effects_list.remove(index);
        }
        despawn_entity(scene, entity);
    }
}
