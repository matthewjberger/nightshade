use crate::ecs::{EffectType, GAME, TowerDefenseResources, VISUAL_EFFECT, VisualEffect};
use crate::systems::despawn_entity;
use nightshade::ecs::material::resources::material_registry_insert;
use nightshade::prelude::*;

pub fn create_explosion_effect(
    game_world: &mut TowerDefenseResources,
    world: &mut World,
    position: Vec3,
) {
    for _ in 0..8 {
        let offset = nalgebra_glm::vec3(
            rand::rng().random_range(-0.5..0.5),
            rand::rng().random_range(0.0..0.5),
            rand::rng().random_range(-0.5..0.5),
        );

        let particle = spawn_mesh(
            world,
            "Sphere",
            position + offset,
            nalgebra_glm::vec3(0.15, 0.15, 0.15),
        );

        let material_name = format!("ExplosionParticle_{}", particle.id);
        material_registry_insert(
            &mut world.resources.assets.material_registry,
            material_name.clone(),
            Material {
                base_color: [1.0, 0.5, 0.0, 0.8],
                emissive_factor: [1.0, 0.3, 0.0],
                alpha_mode: AlphaMode::Blend,
                ..Default::default()
            },
        );
        if let Some(&index) = world
            .resources
            .assets
            .material_registry
            .registry
            .name_to_index
            .get(&material_name)
        {
            registry_add_reference(
                &mut world.resources.assets.material_registry.registry,
                index,
            );
        }
        world.ecs.worlds[CORE].set(particle, MaterialRef::new(material_name));

        world.ecs.worlds[GAME].set(
            particle,
            VisualEffect {
                effect_type: EffectType::Explosion,
                lifetime: 0.5,
                age: 0.0,
            },
        );
        game_world.effects_list.push(particle);
    }
}

pub fn create_death_effect(world: &mut World, position: Vec3) {
    let entity = spawn_entities(world, nightshade::ecs::PARTICLE_EMITTER, 1)[0];
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
    smoke_emitter.shape = nightshade::render::particles::EmitterShape::Sphere { radius: 0.2 };
    world.ecs.worlds[CORE].set(entity, smoke_emitter);
}

pub fn create_poison_bubble_effect(
    game_world: &mut TowerDefenseResources,
    world: &mut World,
    position: Vec3,
) {
    let offset = nalgebra_glm::vec3(
        rand::rng().random_range(-0.2..0.2),
        rand::rng().random_range(0.0..0.5),
        rand::rng().random_range(-0.2..0.2),
    );

    let particle = spawn_mesh(
        world,
        "Sphere",
        position + offset,
        nalgebra_glm::vec3(0.08, 0.08, 0.08),
    );

    let material_name = format!("PoisonBubble_{}", particle.id);
    material_registry_insert(
        &mut world.resources.assets.material_registry,
        material_name.clone(),
        Material {
            base_color: [0.5, 0.0, 0.8, 0.6],
            alpha_mode: AlphaMode::Blend,
            ..Default::default()
        },
    );
    if let Some(&index) = world
        .resources
        .assets
        .material_registry
        .registry
        .name_to_index
        .get(&material_name)
    {
        registry_add_reference(
            &mut world.resources.assets.material_registry.registry,
            index,
        );
    }
    world.ecs.worlds[CORE].set(particle, MaterialRef::new(material_name));

    world.ecs.worlds[GAME].set(
        particle,
        VisualEffect {
            effect_type: EffectType::PoisonBubble,
            lifetime: 2.0,
            age: 0.0,
        },
    );
    game_world.effects_list.push(particle);
}

pub fn create_muzzle_flash(
    game_world: &mut TowerDefenseResources,
    world: &mut World,
    position: Vec3,
) {
    let flash = spawn_mesh(world, "Sphere", position, nalgebra_glm::vec3(0.6, 0.6, 0.6));

    let material_name = format!("MuzzleFlash_{}", flash.id);
    material_registry_insert(
        &mut world.resources.assets.material_registry,
        material_name.clone(),
        Material {
            base_color: [1.0, 1.0, 0.0, 1.0],
            emissive_factor: [2.0, 2.0, 0.0],
            ..Default::default()
        },
    );
    if let Some(&index) = world
        .resources
        .assets
        .material_registry
        .registry
        .name_to_index
        .get(&material_name)
    {
        registry_add_reference(
            &mut world.resources.assets.material_registry.registry,
            index,
        );
    }
    world.ecs.worlds[CORE].set(flash, MaterialRef::new(material_name));

    world.ecs.worlds[GAME].set(
        flash,
        VisualEffect {
            effect_type: EffectType::Explosion,
            lifetime: 0.15,
            age: 0.0,
        },
    );
    game_world.effects_list.push(flash);

    for index in 0..8 {
        let angle = (index as f32 / 8.0) * std::f32::consts::TAU;
        let offset = nalgebra_glm::vec3(angle.cos() * 0.3, 0.0, angle.sin() * 0.3);

        let smoke = spawn_mesh(
            world,
            "Sphere",
            position + offset,
            nalgebra_glm::vec3(0.4, 0.4, 0.4),
        );

        let material_name = format!("MuzzleSmoke_{}", smoke.id);
        material_registry_insert(
            &mut world.resources.assets.material_registry,
            material_name.clone(),
            Material {
                base_color: [0.3, 0.3, 0.3, 0.8],
                alpha_mode: AlphaMode::Blend,
                ..Default::default()
            },
        );
        if let Some(&index) = world
            .resources
            .assets
            .material_registry
            .registry
            .name_to_index
            .get(&material_name)
        {
            registry_add_reference(
                &mut world.resources.assets.material_registry.registry,
                index,
            );
        }
        world.ecs.worlds[CORE].set(smoke, MaterialRef::new(material_name));

        world.ecs.worlds[GAME].set(
            smoke,
            VisualEffect {
                effect_type: EffectType::Explosion,
                lifetime: 2.0,
                age: 0.0,
            },
        );
        game_world.effects_list.push(smoke);
    }
}

pub fn update_visual_effects(
    game_world: &mut TowerDefenseResources,
    world: &mut World,
    delta_time: f32,
) {
    let entities: Vec<Entity> = world.ecs.worlds[GAME]
        .query_entities(VISUAL_EFFECT)
        .collect();
    let mut effects_to_remove = Vec::new();

    for entity in entities {
        if let Some(mut effect) = world.ecs.worlds[GAME].get::<VisualEffect>(entity).copied() {
            effect.age += delta_time;

            if effect.age >= effect.lifetime {
                effects_to_remove.push(entity);
                continue;
            }

            world.ecs.worlds[GAME].set(entity, effect);

            match effect.effect_type {
                EffectType::Explosion => {
                    let progress = effect.age / effect.lifetime;
                    let scale_factor = 1.0 + progress * 2.0;

                    if let Some(transform) = world.ecs.worlds[CORE]
                        .get_mut::<nightshade::ecs::transform::components::LocalTransform>(
                        entity,
                    ) {
                        transform.scale = nalgebra_glm::vec3(
                            0.15 * scale_factor,
                            0.15 * scale_factor,
                            0.15 * scale_factor,
                        );
                    }

                    if let Some(material_ref) = world.ecs.worlds[CORE]
                        .get::<nightshade::ecs::material::components::MaterialRef>(entity)
                        .cloned()
                    {
                        material_registry_mutate(
                            &mut world.resources.assets.material_registry,
                            &material_ref.name,
                            |material| {
                                let alpha = 1.0 - progress;
                                material.base_color[3] = alpha * 0.8;
                            },
                        );
                    }
                }
                EffectType::PoisonBubble => {
                    if let Some(transform) = world.ecs.worlds[CORE]
                        .get_mut::<nightshade::ecs::transform::components::LocalTransform>(
                        entity,
                    ) {
                        transform.translation.y += delta_time * 0.5;
                    }

                    if let Some(material_ref) = world.ecs.worlds[CORE]
                        .get::<nightshade::ecs::material::components::MaterialRef>(entity)
                        .cloned()
                    {
                        material_registry_mutate(
                            &mut world.resources.assets.material_registry,
                            &material_ref.name,
                            |material| {
                                let progress = effect.age / effect.lifetime;
                                let alpha = 1.0 - progress;
                                material.base_color[3] = alpha * 0.6;
                            },
                        );
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
        despawn_entity(world, entity);
    }
}
