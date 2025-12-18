use crate::ecs::{ENTITY_HANDLE, EffectType, EntityHandle, GameWorld, VISUAL_EFFECT, VisualEffect};
use crate::systems::despawn_entity;
use nightshade::ecs::generational_registry::registry_entry_by_name_mut;
use nightshade::ecs::material::resources::material_registry_insert;
use nightshade::prelude::*;

pub fn create_explosion_effect(game_world: &mut GameWorld, world: &mut World, position: Vec3) {
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
            &mut world.resources.material_registry,
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
            .material_registry
            .registry
            .name_to_index
            .get(&material_name)
        {
            world
                .resources
                .material_registry
                .registry
                .add_reference(index);
        }
        world.set_material_ref(particle, MaterialRef::new(material_name));

        let game_entity = game_world.spawn_entities(ENTITY_HANDLE | VISUAL_EFFECT, 1)[0];
        game_world.set_entity_handle(game_entity, EntityHandle(particle));
        game_world.set_visual_effect(
            game_entity,
            VisualEffect {
                effect_type: EffectType::Explosion,
                lifetime: 0.5,
                age: 0.0,
            },
        );
        game_world.resources.effects_list.push(game_entity);
    }
}

pub fn create_death_effect(_game_world: &mut GameWorld, world: &mut World, position: Vec3) {
    let entity = world.spawn_entities(nightshade::ecs::PARTICLE_EMITTER, 1)[0];
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
    smoke_emitter.shape =
        nightshade::ecs::particles::components::EmitterShape::Sphere { radius: 0.2 };
    world.set_particle_emitter(entity, smoke_emitter);
}

pub fn create_poison_bubble_effect(game_world: &mut GameWorld, world: &mut World, position: Vec3) {
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
        &mut world.resources.material_registry,
        material_name.clone(),
        Material {
            base_color: [0.5, 0.0, 0.8, 0.6],
            alpha_mode: AlphaMode::Blend,
            ..Default::default()
        },
    );
    if let Some(&index) = world
        .resources
        .material_registry
        .registry
        .name_to_index
        .get(&material_name)
    {
        world
            .resources
            .material_registry
            .registry
            .add_reference(index);
    }
    world.set_material_ref(particle, MaterialRef::new(material_name));

    let game_entity = game_world.spawn_entities(ENTITY_HANDLE | VISUAL_EFFECT, 1)[0];
    game_world.set_entity_handle(game_entity, EntityHandle(particle));
    game_world.set_visual_effect(
        game_entity,
        VisualEffect {
            effect_type: EffectType::PoisonBubble,
            lifetime: 2.0,
            age: 0.0,
        },
    );
    game_world.resources.effects_list.push(game_entity);
}

pub fn create_muzzle_flash(game_world: &mut GameWorld, world: &mut World, position: Vec3) {
    let flash = spawn_mesh(world, "Sphere", position, nalgebra_glm::vec3(0.6, 0.6, 0.6));

    let material_name = format!("MuzzleFlash_{}", flash.id);
    material_registry_insert(
        &mut world.resources.material_registry,
        material_name.clone(),
        Material {
            base_color: [1.0, 1.0, 0.0, 1.0],
            emissive_factor: [2.0, 2.0, 0.0],
            ..Default::default()
        },
    );
    if let Some(&index) = world
        .resources
        .material_registry
        .registry
        .name_to_index
        .get(&material_name)
    {
        world
            .resources
            .material_registry
            .registry
            .add_reference(index);
    }
    world.set_material_ref(flash, MaterialRef::new(material_name));

    let game_entity = game_world.spawn_entities(ENTITY_HANDLE | VISUAL_EFFECT, 1)[0];
    game_world.set_entity_handle(game_entity, EntityHandle(flash));
    game_world.set_visual_effect(
        game_entity,
        VisualEffect {
            effect_type: EffectType::Explosion,
            lifetime: 0.15,
            age: 0.0,
        },
    );
    game_world.resources.effects_list.push(game_entity);

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
            &mut world.resources.material_registry,
            material_name.clone(),
            Material {
                base_color: [0.3, 0.3, 0.3, 0.8],
                alpha_mode: AlphaMode::Blend,
                ..Default::default()
            },
        );
        if let Some(&index) = world
            .resources
            .material_registry
            .registry
            .name_to_index
            .get(&material_name)
        {
            world
                .resources
                .material_registry
                .registry
                .add_reference(index);
        }
        world.set_material_ref(smoke, MaterialRef::new(material_name));

        let smoke_entity = game_world.spawn_entities(ENTITY_HANDLE | VISUAL_EFFECT, 1)[0];
        game_world.set_entity_handle(smoke_entity, EntityHandle(smoke));
        game_world.set_visual_effect(
            smoke_entity,
            VisualEffect {
                effect_type: EffectType::Explosion,
                lifetime: 2.0,
                age: 0.0,
            },
        );
        game_world.resources.effects_list.push(smoke_entity);
    }
}

pub fn update_visual_effects(game_world: &mut GameWorld, world: &mut World, delta_time: f32) {
    let entities: Vec<_> = game_world.query_entities(VISUAL_EFFECT).collect();
    let mut effects_to_remove = Vec::new();

    for entity in entities {
        if let Some(mut effect) = game_world.get_visual_effect(entity).copied() {
            effect.age += delta_time;

            if effect.age >= effect.lifetime {
                effects_to_remove.push(entity);
                continue;
            }

            game_world.set_visual_effect(entity, effect);

            if let Some(handle) = game_world.get_entity_handle(entity) {
                match effect.effect_type {
                    EffectType::Explosion => {
                        let progress = effect.age / effect.lifetime;
                        let scale_factor = 1.0 + progress * 2.0;

                        if let Some(transform) = world.get_local_transform_mut(handle.0) {
                            transform.scale = nalgebra_glm::vec3(
                                0.15 * scale_factor,
                                0.15 * scale_factor,
                                0.15 * scale_factor,
                            );
                            world.set_local_transform_dirty(handle.0, LocalTransformDirty);
                        }

                        if let Some(material_ref) = world.get_material_ref(handle.0).cloned()
                            && let Some(material) = registry_entry_by_name_mut(
                                &mut world.resources.material_registry.registry,
                                &material_ref.name,
                            )
                        {
                            let alpha = 1.0 - progress;
                            material.base_color[3] = alpha * 0.8;
                        }
                    }
                    EffectType::PoisonBubble => {
                        if let Some(transform) = world.get_local_transform_mut(handle.0) {
                            transform.translation.y += delta_time * 0.5;
                            world.set_local_transform_dirty(handle.0, LocalTransformDirty);
                        }

                        if let Some(material_ref) = world.get_material_ref(handle.0).cloned()
                            && let Some(material) = registry_entry_by_name_mut(
                                &mut world.resources.material_registry.registry,
                                &material_ref.name,
                            )
                        {
                            let progress = effect.age / effect.lifetime;
                            let alpha = 1.0 - progress;
                            material.base_color[3] = alpha * 0.6;
                        }
                    }
                }
            }
        }
    }

    for entity in effects_to_remove {
        if let Some(idx) = game_world
            .resources
            .effects_list
            .iter()
            .position(|&e| e == entity)
        {
            game_world.resources.effects_list.remove(idx);
        }
        despawn_entity(game_world, world, entity);
    }
}
