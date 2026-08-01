use crate::resources::{Script, Store, Vault};
use nightshade::ecs::Storage;
use nightshade::ecs::system_param::{Res, ResMut};
use nightshade_gltf::Loading;
use nightshade_load::{Source, load, take};
use nightshade_prefab::{PrefabNode, PrefabSpawn, queue_model_load};
use nightshade_render::commands::material_registry::material_registry_find_or_insert;
use nightshade_render::commands::texture_cache::texture_cache_acquire;
use nightshade_render::components::material::MaterialRef;
use nightshade_render::components::mesh::RenderMesh;
use nightshade_render::data::generational_registry::registry_add_reference;
use nightshade_render::data::material::Material;
use nightshade_render::resources::material_registry::MaterialRegistry;
use nightshade_render::resources::texture_cache::{TextureCache, TextureOwner};
use nightshade_scene::contract::primitives::Visibility;
use nightshade_scene::contract::{GlobalTransform, LocalTransform};

pub fn fetch(
    script: Res<Script>,
    mut vault: ResMut<Vault>,
    mut loading: ResMut<Loading>,
    mut parts: nightshade_prefab::PrefabParts,
    storage: &mut nightshade::ecs::Storage,
) {
    let models = script.models.clone();
    for path in &models {
        if vault.prefabs.contains_key(path)
            || vault.lost.contains(path)
            || vault.asked.iter().any(|(held, _)| held == path)
        {
            continue;
        }
        let handle = load(&mut loading, Source::Path(std::path::PathBuf::from(path)));
        vault.asked.push((path.clone(), handle));
    }

    let asked = vault.asked.clone();
    for (path, handle) in asked {
        let Some(outcome) = take(&mut loading, handle) else {
            continue;
        };
        vault.asked.retain(|(held, _)| held != &path);
        match outcome {
            Ok(mut import) => {
                let prefab = import.prefabs.first().cloned();
                let mut spawn = PrefabSpawn {
                    storage,
                    parts: parts.reborrow(),
                };
                queue_model_load(&mut spawn, &mut import);
                match prefab {
                    Some(prefab) => {
                        vault.prefabs.insert(path, prefab);
                    }
                    None => {
                        vault.lost.insert(path);
                    }
                }
            }
            Err(_) => {
                vault.lost.insert(path);
            }
        }
    }
}

fn parts(node: &PrefabNode, found: &mut Vec<(RenderMesh, Option<Material>)>) {
    if let Some(mesh) = &node.components.render_mesh {
        found.push((mesh.clone(), node.components.material.clone()));
    }
    for child in &node.children {
        parts(child, found);
    }
}

pub fn raise(
    mut store: ResMut<Store>,
    vault: Res<Vault>,
    mut texture_cache: ResMut<TextureCache>,
    mut material_registry: ResMut<MaterialRegistry>,
    storage: &mut Storage,
) {
    let pending = std::mem::take(&mut store.pending);
    for (row, path) in pending {
        let Some(prefab) = vault.prefabs.get(&path).cloned() else {
            continue;
        };
        let Some(holder) = store.rows.get(row as usize).copied() else {
            continue;
        };
        let mut found = Vec::new();
        for node in &prefab.root_nodes {
            parts(node, &mut found);
        }
        for (index, (mesh, material)) in found.into_iter().enumerate() {
            let entity = match index {
                0 => holder,
                _ => {
                    let made = storage.spawn_entities::<(LocalTransform, GlobalTransform)>(1)[0];
                    storage.set(made, LocalTransform::default());
                    storage.set(made, GlobalTransform::default());
                    storage.set(made, nightshade::ecs::ChildOf(holder));
                    made
                }
            };
            storage.set(entity, mesh);
            storage.set(entity, Visibility { visible: true });
            storage.set(entity, nightshade_render::components::CastsShadow);
            let Some(material) = material else {
                continue;
            };
            let names: Vec<String> = material.texture_names().map(str::to_string).collect();
            texture_cache_acquire(
                &mut texture_cache,
                TextureOwner::EntityMaterial(entity),
                names,
            );
            let held = material_registry_find_or_insert(
                &mut material_registry,
                format!("dsl_model_{}_{index}", entity.id),
                material,
            );
            registry_add_reference(&mut material_registry.registry, held.index);
            storage.set(entity, MaterialRef::new(held));
        }
    }
}
