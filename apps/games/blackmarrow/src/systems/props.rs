use crate::components::{Handle, Heft};
use crate::contract::Fixture;
use crate::data::props::{
    LAMPS, MODEL_SCALE, MODELS, ModelKind, PLACEMENTS, centering, extents, model,
};
use crate::data::theme;
use crate::resources::BlackmarrowResources;
use crate::tags::{Lamp, Station};
use nightshade::ecs::Storage;
use nightshade::prelude::{ResMut, Vec3, nalgebra_glm};
use nightshade_gltf::Loading;
use nightshade_load::{Source, load, take};
use nightshade_physics::contract::components::{ColliderComponent, RigidBodyComponent};
use nightshade_prefab::{PrefabSpawn, queue_model_load, spawn_prefab};
use nightshade_render::components::light::{Light, LightType};
use nightshade_scene::prelude::{ChildOf, GlobalTransform, LocalTransform, Name};

const WOLF_BYTES: &[u8] = include_bytes!("../../assets/creature/wolf.gltf");

pub fn request(mut resources: ResMut<BlackmarrowResources>, mut loading: ResMut<Loading>) {
    if !resources.models.handles.is_empty() {
        return;
    }
    resources.models.prefabs = vec![None; MODELS.len()];
    resources.models.handles = MODELS
        .iter()
        .map(|entry| load(&mut loading, Source::Bytes(entry.bytes.to_vec())))
        .collect();
    resources.models.wolf_handle = Some(load(&mut loading, Source::Bytes(WOLF_BYTES.to_vec())));
}

pub fn collect(
    mut resources: ResMut<BlackmarrowResources>,
    mut loading: ResMut<Loading>,
    mut parts: nightshade_prefab::PrefabParts,
    storage: &mut Storage,
) {
    if resources.models.ready {
        return;
    }
    let handles = resources.models.handles.clone();
    if handles.is_empty() {
        return;
    }

    for (index, handle) in handles.into_iter().enumerate() {
        if resources
            .models
            .prefabs
            .get(index)
            .and_then(|slot| slot.as_ref())
            .is_some()
        {
            continue;
        }
        let Some(Ok(mut import)) = take(&mut loading, handle) else {
            continue;
        };
        let prefab = import.prefabs.first().cloned();
        let mut spawn = PrefabSpawn {
            storage,
            parts: parts.reborrow(),
        };
        queue_model_load(&mut spawn, &mut import);
        resources.models.prefabs[index] = prefab;
    }

    let ready = resources.models.prefabs.iter().all(Option::is_some);
    if ready {
        resources.models.ready = true;
    }
}

pub fn dress(
    mut resources: ResMut<BlackmarrowResources>,
    mut parts: nightshade_prefab::PrefabParts,
    storage: &mut Storage,
) {
    {
        let state = &resources.models;
        if !state.ready || state.dressed {
            return;
        }
    }
    resources.models.dressed = true;

    let prefabs = resources.models.prefabs.clone();

    for placement in PLACEMENTS {
        let Some(index) = MODELS
            .iter()
            .position(|entry| entry.kind == placement.model)
        else {
            continue;
        };
        let Some(Some(prefab)) = prefabs.get(index) else {
            continue;
        };
        let entry = model(placement.model);
        let size = extents(entry);
        let half = size * 0.5;
        let center = placement.at + Vec3::new(0.0, half.y, 0.0);
        let turn = nalgebra_glm::quat_angle_axis(placement.yaw, &Vec3::y());

        let carcass = if placement.loose {
            RigidBodyComponent::new_dynamic()
                .with_translation(center.x, center.y, center.z)
                .with_rotation(turn.i, turn.j, turn.k, turn.w)
                .with_mass(entry.mass)
                .with_linear_damping(0.35)
                .with_angular_damping(0.6)
        } else {
            RigidBodyComponent::new_static()
                .with_translation(center.x, center.y, center.z)
                .with_rotation(turn.i, turn.j, turn.k, turn.w)
        };

        let body = storage.spawn((
            Name(format!("{:?}", placement.model)),
            LocalTransform {
                translation: center,
                rotation: turn,
                scale: Vec3::new(1.0, 1.0, 1.0),
            },
            GlobalTransform::default(),
            carcass,
            ColliderComponent::new_cuboid(half.x, half.y, half.z)
                .with_friction(0.85)
                .with_restitution(0.02),
        ));
        storage.add_tag_type::<Station>(body);
        if placement.loose {
            storage.set(body, Handle(Fixture::Loose));
            storage.set(body, Heft(entry.mass));
        }

        let visual = {
            let mut spawn = PrefabSpawn {
                storage,
                parts: parts.reborrow(),
            };
            spawn_prefab(&mut spawn, prefab, Vec3::zeros())
        };
        if let Some(transform) = storage.get_mut::<LocalTransform>(visual) {
            transform.translation = centering(entry);
            transform.rotation = nalgebra_glm::quat_identity();
            transform.scale = Vec3::new(MODEL_SCALE, MODEL_SCALE, MODEL_SCALE);
        }
        storage.set(visual, ChildOf(body));
    }

    hang_lamps(parts, storage, &prefabs);
}

fn hang_lamps(
    mut parts: nightshade_prefab::PrefabParts<'_>,
    storage: &mut Storage,
    prefabs: &[Option<nightshade_prefab::Prefab>],
) {
    let Some(index) = MODELS
        .iter()
        .position(|entry| entry.kind == ModelKind::WallLamp)
    else {
        return;
    };
    let Some(Some(prefab)) = prefabs.get(index) else {
        return;
    };
    let entry = model(ModelKind::WallLamp);

    for mount in LAMPS {
        let turn = nalgebra_glm::quat_angle_axis(mount.yaw, &Vec3::y());
        let post = storage.spawn((
            Name("Wall Lamp".to_string()),
            LocalTransform {
                translation: mount.at,
                rotation: turn,
                scale: Vec3::new(1.0, 1.0, 1.0),
            },
            GlobalTransform::default(),
        ));
        storage.add_tag_type::<Station>(post);

        let visual = {
            let mut spawn = PrefabSpawn {
                storage,
                parts: parts.reborrow(),
            };
            spawn_prefab(&mut spawn, prefab, Vec3::zeros())
        };
        if let Some(transform) = storage.get_mut::<LocalTransform>(visual) {
            transform.translation = centering(entry);
            transform.rotation = nalgebra_glm::quat_identity();
            transform.scale = Vec3::new(MODEL_SCALE, MODEL_SCALE, MODEL_SCALE);
        }
        storage.set(visual, ChildOf(post));

        let glow = storage.spawn((
            Name("Lamp Light".to_string()),
            LocalTransform {
                translation: Vec3::new(0.0, -0.1, 0.42),
                ..Default::default()
            },
            GlobalTransform::default(),
            Light {
                light_type: LightType::Point,
                color: theme::LAMP_COLOR,
                intensity: theme::LAMP_INTENSITY,
                range: theme::LAMP_RANGE,
                cast_shadows: true,
                shadow_bias: theme::LAMP_SHADOW_BIAS,
                shadow_softness: theme::LAMP_SHADOW_SOFTNESS,
                ..Default::default()
            },
        ));
        storage.set(glow, ChildOf(post));
        storage.add_tag_type::<Lamp>(glow);
        storage.set(glow, nightshade_flicker::FlickerBase(theme::LAMP_INTENSITY));
        storage.set(glow, nightshade_flicker::Flickering(1.0));
    }
}
