use nightshade::prelude::*;
use nightshade_scene::contract::LocalTransform;
use serde::de::DeserializeOwned;
use serde_json::{Value, json};

use crate::data::{
    ALPHA_LABELS, BrowserTab, CameraArgs, ClipArgs, Empty, EntityArgs, ExportArgs, GizmoArgs,
    GizmoMode, LIGHT_KIND_LABELS, LayerArgs, LightArgs, LightKind, LightWriteArgs, MaterialArgs,
    NameArgs, PageArgs, PathArgs, PrimitiveArgs, PrimitiveKind, RenameArgs, RenderArgs, RowKind,
    ScreenshotArgs, SelectArgs, SlugArgs, TransformArgs, VisibleArgs,
};
use crate::queries::material::view_by_name;
use crate::queries::picking::find_entity_by_id;
use crate::queries::selection::{light_view, quat_to_euler};
use crate::resources::ViewerResources;

pub fn json_header() -> tiny_http::Header {
    tiny_http::Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..])
        .unwrap_or_else(|_| tiny_http::Header::from_bytes(&b"X"[..], &b"y"[..]).expect("a header"))
}

pub fn rpc_result(id: Option<Value>, value: Value) -> Value {
    json!({ "jsonrpc": "2.0", "id": id, "result": value })
}

pub fn rpc_error(id: Option<Value>, code: i32, message: &str) -> Value {
    json!({ "jsonrpc": "2.0", "id": id, "error": { "code": code, "message": message } })
}

pub fn tool_list() -> Vec<Value> {
    use enum2schema::mcp::tool;
    vec![
        tool::<Empty>(
            "get_viewer_state",
            "Read which panels are open, the render settings, the handle mode, and the camera in use.",
        ),
        tool::<Empty>(
            "get_scene",
            "Read the scene tree: a number, a name, a kind, a depth, and a parent for each node.",
        ),
        tool::<Empty>(
            "get_stats",
            "Read the counts the loaded model reported: meshes, vertices, triangles, materials, textures, and its size.",
        ),
        tool::<Empty>(
            "get_selection",
            "Read what is selected now, with its transform and the name of its material.",
        ),
        tool::<SelectArgs>(
            "select_entity",
            "Select a node by its number, or leave the entity out to select nothing.",
        ),
        tool::<EntityArgs>(
            "get_transform",
            "Read one node's position, rotation in degrees, and scale.",
        ),
        tool::<TransformArgs>(
            "set_transform",
            "Write a node's position, rotation in degrees, or scale. What is left out is not changed.",
        ),
        tool::<EntityArgs>(
            "get_material",
            "Read the material of a node, and the textures it names.",
        ),
        tool::<MaterialArgs>(
            "set_material",
            "Write values onto the material a node names. Every mesh naming that material changes with it.",
        ),
        tool::<Empty>(
            "list_materials",
            "Read the name of every material the loaded model brought.",
        ),
        tool::<EntityArgs>("delete_entity", "Delete a node and everything under it."),
        tool::<PrimitiveArgs>(
            "add_primitive",
            "Add a cube, sphere, cylinder, cone, torus, or plane where the camera is looking.",
        ),
        tool::<LightArgs>(
            "add_light",
            "Add a directional, point, or spot light where the camera is looking.",
        ),
        tool::<PathArgs>(
            "load_file",
            "Load a glb, gltf, hdr, or zip file from this machine.",
        ),
        tool::<NameArgs>("load_khronos", "Load a Khronos sample model by name."),
        tool::<SlugArgs>("load_polyhaven_model", "Load a Polyhaven model by slug."),
        tool::<SlugArgs>("load_polyhaven_sky", "Load a Polyhaven sky by slug."),
        tool::<PageArgs>(
            "list_library",
            "Read what one library tab offers, with an optional search over the names and a page of sixty.",
        ),
        tool::<ScreenshotArgs>(
            "screenshot",
            "Write the next frame to a file on this machine, or ask the reader where when no path is given.",
        ),
        tool::<Empty>(
            "apply_clips",
            "Put the clips of the model that was open before onto the skeleton open now, matching joints by name.",
        ),
        tool::<ExportArgs>(
            "export_model",
            "Write what is open now to a glb on this machine, or ask the reader where when no path is given.",
        ),
        tool::<VisibleArgs>(
            "set_visible",
            "Show or hide a node and everything under it.",
        ),
        tool::<RenameArgs>("rename_entity", "Give a node a new name."),
        tool::<SelectArgs>(
            "isolate",
            "Show only one node with what is above and below it, or leave the entity out to show everything again.",
        ),
        tool::<EntityArgs>("get_light", "Read the light a node carries."),
        tool::<LightWriteArgs>(
            "set_light",
            "Write values onto the light a node carries. What is left out is not changed.",
        ),
        tool::<Empty>("undo", "Take the last edit back."),
        tool::<Empty>("redo", "Do the last edit taken back again."),
        tool::<Empty>(
            "get_animation",
            "Read the clips, which one is playing, and the layers over it.",
        ),
        tool::<ClipArgs>(
            "play_clip",
            "Play a clip, blending into it over the time given.",
        ),
        tool::<Empty>(
            "stop_animation",
            "Stop the animation and go back to the start.",
        ),
        tool::<LayerArgs>(
            "add_animation_layer",
            "Lay a clip over the one that is playing.",
        ),
        tool::<LayerArgs>(
            "remove_animation_layer",
            "Take a layer away by its position.",
        ),
        tool::<GizmoArgs>(
            "set_handle_mode",
            "Show the move, rotate, or scale handles, or none to put them away.",
        ),
        tool::<RenderArgs>(
            "set_render",
            "Write render settings. What is left out is not changed.",
        ),
        tool::<CameraArgs>(
            "set_active_camera",
            "Look through a camera the file carried, or leave the entity out for the viewer's own.",
        ),
        tool::<Empty>(
            "frame_model",
            "Move the camera until the whole model is in view.",
        ),
    ]
}

pub fn take_arguments<T: DeserializeOwned + Default>(arguments: Value) -> Result<T, String> {
    if arguments.is_null() {
        return Ok(T::default());
    }
    serde_json::from_value(arguments)
        .map_err(|error| format!("those arguments do not fit: {error}"))
}

pub fn resolve(storage: &nightshade::ecs::Storage, id: u32) -> Result<Entity, String> {
    find_entity_by_id(storage, id).ok_or_else(|| format!("there is no live node numbered {id}"))
}

pub fn viewer_state(stage: &crate::resources::Stage<'_>) -> Value {
    let viewer = &*stage.viewer;
    let render = &*stage.settings;
    let debug = &*stage.debug_draw;
    json!({
        "scene_panel_open": viewer.view.scene_open,
        "inspector_open": viewer.view.inspector_open,
        "library_open": viewer.view.browser.is_some(),
        "interface_visible": viewer.view.interface_visible,
        "grid": debug.show_grid,
        "snap": viewer.view.snap,
        "turntable": viewer.camera_input.turntable,
        "shading": viewer.view.shading,
        "pbr_debug": viewer.view.pbr_debug,
        "tonemap": viewer.view.tonemap,
        "exposure": render.color_grading.exposure,
        "atmosphere": viewer.view.atmosphere,
        "sky": render.show_sky,
        "normals": debug.show_normals,
        "bounds": debug.show_bounding_volumes,
        "skeleton": viewer.skeleton.enabled,
        "ssao": render.ssao_enabled,
        "bloom": render.bloom_enabled,
        "ssr": render.ssr_enabled,
        "ssgi": render.ssgi_enabled,
        "anti_aliasing": viewer.view.anti_aliasing,
        "render_scale": render.render_scale,
        "fog": render.fog.is_some(),
        "depth_of_field": render.depth_of_field.enabled,
        "sun": {
            "intensity": viewer.sun.intensity,
            "yaw": viewer.sun.yaw,
            "pitch": viewer.sun.pitch,
            "shadows": viewer.sun.shadows,
        },
        "isolated": viewer.model.isolated.map(|entity| entity.id),
        "edits_to_undo": viewer.history.done.len(),
        "edits_to_redo": viewer.history.undone.len(),
        "handles": handle_name(viewer),
        "cameras": viewer
            .model
            .cameras
            .iter()
            .map(|(entity, name)| json!({ "entity": entity.id, "name": name }))
            .collect::<Vec<Value>>(),
        "active_camera": stage.active_camera.0.map(|entity| entity.id),
    })
}

fn handle_name(viewer: &ViewerResources) -> Value {
    if !viewer.gizmo.enabled {
        return json!("none");
    }
    json!(match viewer.gizmo.mode {
        GizmoMode::Move => "move",
        GizmoMode::Rotate => "rotate",
        GizmoMode::Scale => "scale",
    })
}

pub fn scene_state(stage: &crate::resources::Stage<'_>) -> Value {
    let viewer = &*stage.viewer;
    json!({
        "nodes": viewer
            .model
            .rows
            .iter()
            .map(|row| json!({
                "entity": row.entity.id,
                "name": row.name,
                "depth": row.depth,
                "visible": row.visible,
                "parent": row.parent.map(|parent| parent.id),
                "kind": match row.kind {
                    RowKind::Mesh => "mesh",
                    RowKind::Light => "light",
                    RowKind::Camera => "camera",
                    RowKind::Node => "node",
                },
            }))
            .collect::<Vec<Value>>(),
    })
}

pub fn stats_state(stage: &crate::resources::Stage<'_>) -> Value {
    let viewer = &*stage.viewer;
    match viewer.model.stats.as_ref() {
        Some(stats) => json!({
            "meshes": stats.meshes,
            "vertices": stats.vertices,
            "triangles": stats.triangles,
            "materials": stats.materials,
            "textures": stats.textures,
            "size": stats.dimensions,
        }),
        None => json!({ "loaded": false }),
    }
}

pub fn selection_state(stage: &crate::resources::Stage<'_>) -> Value {
    let viewer = &*stage.viewer;
    match viewer.selection.selected {
        Some(entity) => json!({
            "entity": entity.id,
            "name": viewer.selection.name,
            "mesh": viewer.selection.mesh,
            "material": viewer.selection.material,
            "translation": viewer.selection.translation,
            "rotation": viewer.selection.rotation,
            "scale": viewer.selection.scale,
        }),
        None => json!({ "entity": Value::Null }),
    }
}

pub fn animation_state(stage: &crate::resources::Stage<'_>) -> Value {
    let viewer = &*stage.viewer;
    json!({
        "clips": viewer
            .model
            .clips
            .iter()
            .enumerate()
            .map(|(index, clip)| json!({
                "index": index,
                "name": clip.name,
                "duration": clip.duration,
            }))
            .collect::<Vec<Value>>(),
        "playing_clip": viewer.animation.clip,
        "playing": viewer.animation.playing,
        "time": viewer.animation.time,
        "duration": viewer.animation.duration,
        "speed": viewer.animation.speed,
        "looping": viewer.animation.looping,
        "blend_time": viewer.animation.blend_time,
        "blending": viewer.animation.blending,
        "layers": viewer
            .animation
            .layers
            .iter()
            .map(|layer| json!({
                "clip": layer.clip,
                "name": layer.name,
                "weight": layer.weight,
            }))
            .collect::<Vec<Value>>(),
    })
}

pub fn material_names(stage: &crate::resources::Stage<'_>) -> Value {
    let registry = &*stage.materials;
    json!({
        "materials": nightshade_render::queries::material_registry::material_registry_iter(
            registry,
        )
        .map(|(name, _)| Value::String(name.clone()))
        .collect::<Vec<Value>>(),
    })
}

pub fn transform_of(storage: &nightshade::ecs::Storage, entity: Entity) -> Result<Value, String> {
    let transform = storage
        .get::<LocalTransform>(entity)
        .ok_or("that node carries no transform")?;
    Ok(json!({
        "entity": entity.id,
        "translation": [transform.translation.x, transform.translation.y, transform.translation.z],
        "rotation": quat_to_euler(transform.rotation),
        "scale": [transform.scale.x, transform.scale.y, transform.scale.z],
    }))
}

pub fn material_of(
    stage: &crate::resources::Stage<'_>,
    storage: &nightshade::ecs::Storage,
    entity: Entity,
) -> Result<Value, String> {
    use nightshade_render::components::material::MaterialRef;
    let id = storage
        .get::<MaterialRef>(entity)
        .map(|reference| reference.id)
        .ok_or("that node names no material")?;
    let registry = &*stage.materials;
    let name = nightshade_render::queries::material_registry::material_registry_name(registry, id)
        .map(str::to_string)
        .ok_or("that node names no material")?;
    let values = view_by_name(registry, &name)
        .ok_or_else(|| format!("no material is registered as {name}"))?;
    Ok(json!({
        "entity": entity.id,
        "name": name,
        "base_color": values.base_color,
        "metallic": values.metallic,
        "roughness": values.roughness,
        "alpha_mode": ALPHA_LABELS[values.alpha_mode.min(3)],
        "alpha_cutoff": values.alpha_cutoff,
        "unlit": values.unlit,
        "double_sided": values.double_sided,
        "emissive": values.emissive,
        "emissive_strength": values.emissive_strength,
        "ior": values.ior,
        "specular": values.specular,
        "transmission": values.transmission,
        "thickness": values.thickness,
        "textures": values
            .textures
            .iter()
            .map(|(slot, name)| json!({ "slot": slot.label(), "texture": name }))
            .collect::<Vec<Value>>(),
    }))
}

pub fn light_of(storage: &nightshade::ecs::Storage, entity: Entity) -> Result<Value, String> {
    let light = storage
        .get::<nightshade_graphics::prelude::Light>(entity)
        .ok_or("that node carries no light")?;
    let view = light_view(light);
    Ok(json!({
        "entity": entity.id,
        "kind": LIGHT_KIND_LABELS[view.kind.min(2)],
        "color": view.color,
        "intensity": view.intensity,
        "range": view.range,
        "inner_cone": view.inner_cone,
        "outer_cone": view.outer_cone,
        "cast_shadows": view.cast_shadows,
    }))
}

pub fn library(stage: &crate::resources::Stage<'_>, args: &PageArgs) -> Result<Value, String> {
    use crate::data::BrowserItem;
    use crate::queries::browsers::{page_count, page_entries};
    let viewer = &*stage.viewer;
    let query = args.search.clone().unwrap_or_default().to_lowercase();
    let tab = browser_tab(&args.tab)?;
    let pages = page_count(viewer, tab, &query);
    let page = args.page.unwrap_or(0).min(pages - 1);
    let entries: Vec<Value> = page_entries(viewer, tab, &query, page)
        .into_iter()
        .map(|entry| match entry.item {
            BrowserItem::Khronos(name) => json!({ "name": name, "label": entry.label }),
            BrowserItem::Model(slug) | BrowserItem::Sky(slug) => {
                json!({ "slug": slug, "name": entry.label })
            }
            BrowserItem::File(path) => json!({ "path": path, "name": entry.label }),
        })
        .collect();
    Ok(json!({ "page": page, "pages": pages, "entries": entries }))
}

pub fn primitive_kind(kind: &str) -> Result<PrimitiveKind, String> {
    match kind.to_lowercase().as_str() {
        "cube" => Ok(PrimitiveKind::Cube),
        "sphere" => Ok(PrimitiveKind::Sphere),
        "cylinder" => Ok(PrimitiveKind::Cylinder),
        "cone" => Ok(PrimitiveKind::Cone),
        "torus" => Ok(PrimitiveKind::Torus),
        "plane" => Ok(PrimitiveKind::Plane),
        _ => Err("the kind must be cube, sphere, cylinder, cone, torus, or plane".to_string()),
    }
}

pub fn light_kind(kind: &str) -> Result<LightKind, String> {
    match kind.to_lowercase().as_str() {
        "directional" => Ok(LightKind::Directional),
        "point" => Ok(LightKind::Point),
        "spot" => Ok(LightKind::Spot),
        _ => Err("the kind must be directional, point, or spot".to_string()),
    }
}

pub fn browser_tab(name: &str) -> Result<BrowserTab, String> {
    match name.to_lowercase().as_str() {
        "library" => Ok(BrowserTab::Library),
        "khronos" => Ok(BrowserTab::Khronos),
        "models" => Ok(BrowserTab::Models),
        "skies" => Ok(BrowserTab::Skies),
        _ => Err("the tab must be library, khronos, models, or skies".to_string()),
    }
}
