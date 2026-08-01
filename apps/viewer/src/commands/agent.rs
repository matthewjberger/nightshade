use std::sync::mpsc::{Receiver, Sender, channel};

use nightshade::prelude::*;
use nightshade_graphics::prelude::CommandQueues;
use nightshade_scene::contract::LocalTransform;
use serde_json::{Value, json};

use crate::commands::crew::{hire, hold_the_agent};
use crate::commands::files::route_drop;
use crate::commands::material::write_field;
use crate::data::{
    ALPHA_LABELS, CameraArgs, ClipArgs, EntityArgs, ExportArgs, GizmoArgs, GizmoMode, LayerArgs,
    LightArgs, LightField, LightWriteArgs, MaterialArgs, MaterialField, NameArgs, PageArgs,
    PathArgs, PrimitiveArgs, RenameArgs, RenderArgs, ScreenshotArgs, SelectArgs, SlugArgs,
    TransformArgs, ViewerAction, VisibleArgs,
};
use crate::queries::agent::{
    animation_state, json_header, library, light_kind, light_of, material_names, material_of,
    primitive_kind, resolve, rpc_error, rpc_result, scene_state, selection_state, stats_state,
    take_arguments, tool_list, transform_of, viewer_state,
};
use crate::queries::files::read_path;
use crate::queries::selection::euler_to_quat;
use crate::resources::{AgentChannel, Crew, Pending};

pub fn take_pending(channel: &AgentChannel) -> Vec<Pending> {
    channel
        .pending
        .lock()
        .map(|mut queue| std::mem::take(&mut *queue))
        .unwrap_or_default()
}

fn queue_pending(channel: &AgentChannel, pending: Pending) {
    if let Ok(mut queue) = channel.pending.lock() {
        queue.push(pending);
    }
}

pub fn start_agent(crew: &mut Crew, port: u16) -> (AgentChannel, Option<String>) {
    let channel = AgentChannel::default();
    let address = format!("127.0.0.1:{port}");
    let server = match tiny_http::Server::http(&address) {
        Ok(server) => server,
        Err(error) => {
            return (
                channel,
                Some(format!(
                    "The agent endpoint did not open on {address}. {error}"
                )),
            );
        }
    };

    let server = std::sync::Arc::new(server);
    hold_the_agent(crew, std::sync::Arc::clone(&server));
    let queue = channel.clone();
    hire(crew, move || {
        for mut request in server.incoming_requests() {
            let mut body = String::new();
            if std::io::Read::read_to_string(&mut request.as_reader(), &mut body).is_err() {
                continue;
            }
            let answer = answer_message(&queue, &body);
            let response = match answer {
                Some(value) => {
                    tiny_http::Response::from_string(value.to_string()).with_header(json_header())
                }
                None => tiny_http::Response::from_string(String::new())
                    .with_header(json_header())
                    .with_status_code(202),
            };
            if request.respond(response).is_err() {
                continue;
            }
        }
    });

    (channel, None)
}

fn answer_message(queue: &AgentChannel, body: &str) -> Option<Value> {
    let message: Value = serde_json::from_str(body).ok()?;
    let id = message.get("id").cloned();
    let method = message.get("method")?.as_str()?.to_string();
    let params = message.get("params").cloned().unwrap_or(Value::Null);

    match method.as_str() {
        "initialize" => Some(rpc_result(
            id,
            json!({
                "protocolVersion": "2024-11-05",
                "capabilities": { "tools": {} },
                "serverInfo": { "name": "nightshade-viewer", "version": "1" },
            }),
        )),
        "notifications/initialized" => None,
        "tools/list" => Some(rpc_result(id, json!({ "tools": tool_list() }))),
        "tools/call" => Some(queue_call(queue, id, params)),
        _ => Some(rpc_error(id, -32601, "no such method")),
    }
}

fn queue_call(queue: &AgentChannel, id: Option<Value>, params: Value) -> Value {
    let Some(name) = params.get("name").and_then(Value::as_str) else {
        return rpc_error(id, -32602, "the call named no tool");
    };
    let arguments = params
        .get("arguments")
        .cloned()
        .unwrap_or_else(|| json!({}));

    let (sender, receiver): (Sender<Value>, Receiver<Value>) = channel();
    queue_pending(
        queue,
        Pending {
            name: name.to_string(),
            arguments,
            reply: sender,
        },
    );

    match receiver.recv_timeout(std::time::Duration::from_secs(20)) {
        Ok(value) => rpc_result(
            id,
            json!({
                "content": [{ "type": "text", "text": value.to_string() }],
                "structuredContent": value,
                "isError": false,
            }),
        ),
        Err(_) => rpc_error(id, -32000, "the viewer did not answer in time"),
    }
}

pub fn answer_call(
    stage: &mut crate::resources::Stage<'_>,
    storage: &mut nightshade::ecs::Storage,
    name: &str,
    arguments: Value,
) -> Result<Value, String> {
    match name {
        "get_viewer_state" => Ok(viewer_state(stage)),
        "get_scene" => Ok(scene_state(stage)),
        "get_stats" => Ok(stats_state(stage)),
        "get_selection" => Ok(selection_state(stage)),
        "get_animation" => Ok(animation_state(stage)),
        "list_materials" => Ok(material_names(stage)),

        "select_entity" => {
            let args: SelectArgs = take_arguments(arguments)?;
            let entity = args.entity.map(|id| resolve(storage, id)).transpose()?;
            push_action(stage, ViewerAction::Select(entity));
            Ok(json!({ "selected": args.entity }))
        }
        "get_transform" => {
            let args: EntityArgs = take_arguments(arguments)?;
            let entity = resolve(storage, args.entity)?;
            transform_of(storage, entity)
        }
        "set_transform" => {
            let args: TransformArgs = take_arguments(arguments)?;
            let entity = resolve(storage, args.entity)?;
            write_transform(storage, entity, &args)?;
            transform_of(storage, entity)
        }
        "get_material" => {
            let args: EntityArgs = take_arguments(arguments)?;
            let entity = resolve(storage, args.entity)?;
            material_of(stage, storage, entity)
        }
        "set_material" => {
            let args: MaterialArgs = take_arguments(arguments)?;
            let entity = resolve(storage, args.entity)?;
            write_material(stage, storage, entity, &args)?;
            material_of(stage, storage, entity)
        }
        "delete_entity" => {
            let args: EntityArgs = take_arguments(arguments)?;
            let entity = resolve(storage, args.entity)?;
            push_action(stage, ViewerAction::Select(Some(entity)));
            push_action(stage, ViewerAction::DeleteSelected);
            Ok(json!({ "deleted": args.entity }))
        }
        "add_primitive" => {
            let args: PrimitiveArgs = take_arguments(arguments)?;
            push_action(
                stage,
                ViewerAction::AddPrimitive(primitive_kind(&args.kind)?),
            );
            Ok(json!({ "added": args.kind }))
        }
        "add_light" => {
            let args: LightArgs = take_arguments(arguments)?;
            push_action(stage, ViewerAction::AddLight(light_kind(&args.kind)?));
            Ok(json!({ "added": args.kind }))
        }
        "load_file" => {
            let args: PathArgs = take_arguments(arguments)?;
            load_file(stage, &args.path)
        }
        "load_khronos" => {
            let args: NameArgs = take_arguments(arguments)?;
            push_action(stage, ViewerAction::LoadKhronos(args.name.clone()));
            Ok(json!({ "loading": args.name }))
        }
        "load_polyhaven_model" => {
            let args: SlugArgs = take_arguments(arguments)?;
            with_resolution(stage, args.resolution);
            push_action(stage, ViewerAction::LoadPolyhavenModel(args.slug.clone()));
            Ok(json!({ "loading": args.slug }))
        }
        "load_polyhaven_sky" => {
            let args: SlugArgs = take_arguments(arguments)?;
            with_resolution(stage, args.resolution);
            push_action(stage, ViewerAction::LoadPolyhavenSky(args.slug.clone()));
            Ok(json!({ "loading": args.slug }))
        }
        "list_library" => {
            let args: PageArgs = take_arguments(arguments)?;
            library(stage, &args)
        }
        "screenshot" => {
            let args: ScreenshotArgs = take_arguments(arguments)?;
            match args.path {
                Some(path) => screenshot_to(stage, &path),
                None => {
                    push_action(stage, ViewerAction::Screenshot);
                    Ok(json!({ "asking": true }))
                }
            }
        }
        "apply_clips" => {
            push_action(stage, ViewerAction::ApplyHeldClips);
            Ok(json!({ "asking": true }))
        }
        "export_model" => {
            let args: ExportArgs = take_arguments(arguments)?;
            match args.path {
                Some(path) => export_to(stage, storage, &path),
                None => {
                    push_action(stage, ViewerAction::ExportModel);
                    Ok(json!({ "asking": true }))
                }
            }
        }
        "set_visible" => {
            let args: VisibleArgs = take_arguments(arguments)?;
            let entity = resolve(storage, args.entity)?;
            push_action(
                stage,
                ViewerAction::SetNodeVisible {
                    entity,
                    visible: args.visible,
                },
            );
            Ok(json!({ "entity": args.entity, "visible": args.visible }))
        }
        "rename_entity" => {
            let args: RenameArgs = take_arguments(arguments)?;
            let entity = resolve(storage, args.entity)?;
            push_action(stage, ViewerAction::Select(Some(entity)));
            push_action(stage, ViewerAction::Rename(args.name.clone()));
            Ok(json!({ "entity": args.entity, "name": args.name }))
        }
        "isolate" => {
            let args: SelectArgs = take_arguments(arguments)?;
            let entity = args.entity.map(|id| resolve(storage, id)).transpose()?;
            let isolated = stage.viewer.model.isolated;
            match entity {
                Some(entity) => {
                    if isolated.is_some() {
                        push_action(stage, ViewerAction::ToggleIsolate);
                    }
                    push_action(stage, ViewerAction::Select(Some(entity)));
                    push_action(stage, ViewerAction::ToggleIsolate);
                }
                None if isolated.is_some() => push_action(stage, ViewerAction::ToggleIsolate),
                None => {}
            }
            Ok(json!({ "isolated": args.entity }))
        }
        "get_light" => {
            let args: EntityArgs = take_arguments(arguments)?;
            let entity = resolve(storage, args.entity)?;
            light_of(storage, entity)
        }
        "set_light" => {
            let args: LightWriteArgs = take_arguments(arguments)?;
            let entity = resolve(storage, args.entity)?;
            write_light(stage, storage, entity, &args)?;
            light_of(storage, entity)
        }
        "undo" => {
            push_action(stage, ViewerAction::Undo);
            Ok(json!({ "undone": true }))
        }
        "redo" => {
            push_action(stage, ViewerAction::Redo);
            Ok(json!({ "redone": true }))
        }
        "play_clip" => {
            let args: ClipArgs = take_arguments(arguments)?;
            if let Some(time) = args.blend_time {
                push_action(stage, ViewerAction::SetBlendTime(time));
            }
            push_action(stage, ViewerAction::BlendToClip(args.index));
            Ok(json!({ "playing": args.index }))
        }
        "stop_animation" => {
            push_action(stage, ViewerAction::StopAnimation);
            Ok(json!({ "stopped": true }))
        }
        "add_animation_layer" => {
            let args: LayerArgs = take_arguments(arguments)?;
            push_action(
                stage,
                ViewerAction::AddAnimationLayer {
                    clip: args.index,
                    weight: args.weight.unwrap_or(0.5),
                },
            );
            Ok(json!({ "layered": args.index }))
        }
        "remove_animation_layer" => {
            let args: LayerArgs = take_arguments(arguments)?;
            push_action(stage, ViewerAction::RemoveAnimationLayer(args.index));
            Ok(json!({ "removed": args.index }))
        }
        "set_handle_mode" => {
            let args: GizmoArgs = take_arguments(arguments)?;
            handle_mode(stage, &args.mode)
        }
        "set_render" => {
            let args: RenderArgs = take_arguments(arguments)?;
            apply_render_arguments(stage, &args);
            Ok(viewer_state(stage))
        }
        "set_active_camera" => {
            let args: CameraArgs = take_arguments(arguments)?;
            let entity = args.entity.map(|id| resolve(storage, id)).transpose()?;
            push_action(stage, ViewerAction::SetActiveCamera(entity));
            Ok(json!({ "camera": args.entity }))
        }
        "frame_model" => {
            push_action(stage, ViewerAction::FrameModel);
            Ok(json!({ "framing": true }))
        }
        _ => Err(format!("there is no tool named {name}")),
    }
}

fn push_action(stage: &mut crate::resources::Stage<'_>, action: ViewerAction) {
    stage.viewer.actions.push(action);
}

fn write_transform(
    storage: &mut nightshade::ecs::Storage,
    entity: Entity,
    args: &TransformArgs,
) -> Result<(), String> {
    let transform = storage
        .get_mut::<LocalTransform>(entity)
        .ok_or("that node carries no transform")?;
    if let Some(values) = args.translation {
        transform.translation = Vec3::new(values[0], values[1], values[2]);
    }
    if let Some(values) = args.rotation {
        transform.rotation = euler_to_quat(values);
    }
    if let Some(values) = args.scale {
        transform.scale = Vec3::new(values[0], values[1], values[2]);
    }
    Ok(())
}

fn write_material(
    stage: &mut crate::resources::Stage<'_>,
    storage: &nightshade::ecs::Storage,
    entity: Entity,
    args: &MaterialArgs,
) -> Result<(), String> {
    use nightshade_render::components::material::MaterialRef;
    let id = storage
        .get::<MaterialRef>(entity)
        .map(|reference| reference.id)
        .ok_or("that node names no material")?;
    let name =
        nightshade_render::queries::material_registry::material_registry_name(stage.materials, id)
            .map(str::to_string)
            .ok_or("that node names no material")?;

    let mut fields = Vec::new();
    if let Some(value) = args.base_color {
        fields.push(MaterialField::BaseColor(value));
    }
    if let Some(value) = args.metallic {
        fields.push(MaterialField::Metallic(value));
    }
    if let Some(value) = args.roughness {
        fields.push(MaterialField::Roughness(value));
    }
    if let Some(value) = args.emissive {
        fields.push(MaterialField::Emissive(value));
    }
    if let Some(value) = args.emissive_strength {
        fields.push(MaterialField::EmissiveStrength(value));
    }
    if let Some(value) = args.ior {
        fields.push(MaterialField::Ior(value));
    }
    if let Some(value) = args.specular {
        fields.push(MaterialField::Specular(value));
    }
    if let Some(value) = args.transmission {
        fields.push(MaterialField::Transmission(value));
    }
    if let Some(value) = args.thickness {
        fields.push(MaterialField::Thickness(value));
    }
    if let Some(value) = args.unlit {
        fields.push(MaterialField::Unlit(value));
    }
    if let Some(value) = args.double_sided {
        fields.push(MaterialField::DoubleSided(value));
    }
    if let Some(mode) = args.alpha_mode.as_deref() {
        let index = ALPHA_LABELS
            .iter()
            .position(|label| label.eq_ignore_ascii_case(mode))
            .ok_or("the alpha mode must be opaque, mask, blend, or dither")?;
        fields.push(MaterialField::AlphaMode(index));
    }

    let registry = &mut *stage.materials;
    for field in fields {
        write_field(registry, &name, field);
    }
    stage.viewer.view.detail_dirty = true;
    Ok(())
}

pub fn write_light(
    stage: &mut crate::resources::Stage<'_>,
    storage: &nightshade::ecs::Storage,
    entity: Entity,
    args: &LightWriteArgs,
) -> Result<(), String> {
    if !storage.has::<nightshade_graphics::prelude::Light>(entity) {
        return Err("that node carries no light".to_string());
    }
    let mut fields = Vec::new();
    if let Some(value) = args.color {
        fields.push(LightField::Color(value));
    }
    if let Some(value) = args.intensity {
        fields.push(LightField::Intensity(value));
    }
    if let Some(value) = args.range {
        fields.push(LightField::Range(value));
    }
    if let Some(value) = args.inner_cone {
        fields.push(LightField::InnerCone(value));
    }
    if let Some(value) = args.outer_cone {
        fields.push(LightField::OuterCone(value));
    }
    if let Some(value) = args.cast_shadows {
        fields.push(LightField::CastShadows(value));
    }
    push_action(stage, ViewerAction::Select(Some(entity)));
    for field in fields {
        push_action(stage, ViewerAction::SetLightField(field));
    }
    Ok(())
}

pub fn load_file(stage: &mut crate::resources::Stage<'_>, path: &str) -> Result<Value, String> {
    let file =
        read_path(std::path::Path::new(path)).ok_or_else(|| format!("{path} could not be read"))?;
    let inbox = stage.viewer.incoming.clone();
    route_drop(&inbox, vec![file]);
    Ok(json!({ "loading": path }))
}

fn with_resolution(stage: &mut crate::resources::Stage<'_>, resolution: Option<u32>) {
    if let Some(value) = resolution {
        stage.viewer.view.resolution = value;
    }
}

fn screenshot_to(stage: &mut crate::resources::Stage<'_>, path: &str) -> Result<Value, String> {
    write_screenshot(stage.queues, path);
    Ok(json!({ "writing": path }))
}

fn write_screenshot(queues: &mut CommandQueues, path: &str) {
    nightshade_graphics::commands::queues::capture_screenshot_to_path(
        queues,
        std::path::PathBuf::from(path),
    );
}

fn handle_mode(stage: &mut crate::resources::Stage<'_>, mode: &str) -> Result<Value, String> {
    let gizmo = &mut stage.viewer.gizmo;
    match mode.to_lowercase().as_str() {
        "none" => gizmo.enabled = false,
        "move" => {
            gizmo.enabled = true;
            gizmo.mode = GizmoMode::Move;
        }
        "rotate" => {
            gizmo.enabled = true;
            gizmo.mode = GizmoMode::Rotate;
        }
        "scale" => {
            gizmo.enabled = true;
            gizmo.mode = GizmoMode::Scale;
        }
        _ => return Err("the mode must be move, rotate, scale, or none".to_string()),
    }
    Ok(json!({ "handles": mode }))
}

fn apply_render_arguments(stage: &mut crate::resources::Stage<'_>, args: &RenderArgs) {
    let mut actions = Vec::new();
    if let Some(value) = args.exposure {
        actions.push(ViewerAction::SetExposure(value));
    }
    if let Some(value) = args.tonemap {
        actions.push(ViewerAction::SetTonemap(value));
    }
    if let Some(value) = args.shading {
        actions.push(ViewerAction::SetShadingMode(value));
    }
    if let Some(value) = args.pbr_debug {
        actions.push(ViewerAction::SetPbrDebug(value));
    }
    if let Some(value) = args.grid {
        actions.push(ViewerAction::SetGrid(value));
    }
    if let Some(value) = args.sky {
        actions.push(ViewerAction::SetShowSky(value));
    }
    if let Some(value) = args.normals {
        actions.push(ViewerAction::SetShowNormals(value));
    }
    if let Some(value) = args.bounds {
        actions.push(ViewerAction::SetShowBounds(value));
    }
    if let Some(value) = args.skeleton {
        actions.push(ViewerAction::SetShowSkeleton(value));
    }
    if let Some(value) = args.turntable {
        actions.push(ViewerAction::SetTurntable(value));
    }
    if let Some(value) = args.atmosphere {
        actions.push(ViewerAction::SetAtmosphere(value));
    }
    if let Some(value) = args.ssao {
        actions.push(ViewerAction::SetSsao(value));
    }
    if let Some(value) = args.bloom {
        actions.push(ViewerAction::SetBloom(value));
    }
    if let Some(value) = args.ssr {
        actions.push(ViewerAction::SetSsr(value));
    }
    if let Some(value) = args.ssgi {
        actions.push(ViewerAction::SetSsgi(value));
    }
    if let Some(value) = args.anti_aliasing {
        actions.push(ViewerAction::SetAntiAliasing(value));
    }
    if let Some(value) = args.render_scale {
        actions.push(ViewerAction::SetRenderScale(value));
    }
    if let Some(value) = args.fog {
        actions.push(ViewerAction::SetFog(value));
    }
    if let Some(value) = args.depth_of_field {
        actions.push(ViewerAction::SetDepthOfField(value));
    }
    if let Some(value) = args.sun_intensity {
        actions.push(ViewerAction::SetSunIntensity(value));
    }
    if let Some(value) = args.sun_yaw {
        actions.push(ViewerAction::SetSunYaw(value));
    }
    if let Some(value) = args.sun_pitch {
        actions.push(ViewerAction::SetSunPitch(value));
    }
    if let Some(value) = args.sun_shadows {
        actions.push(ViewerAction::SetSunShadows(value));
    }
    let viewer = &mut *stage.viewer;
    viewer.actions.extend(actions);
    viewer.view.render_panel_dirty = true;
}

fn export_to(
    stage: &mut crate::resources::Stage<'_>,
    storage: &nightshade::ecs::Storage,
    path: &str,
) -> Result<serde_json::Value, String> {
    let scene = crate::commands::files::gather_scene(stage, storage, path)?;
    let nodes = scene.nodes.len();
    let clips = scene.animations.len();
    let bytes = nightshade_gltf::export_glb(&scene)?;
    std::fs::write(path, &bytes).map_err(|error| error.to_string())?;
    Ok(json!({
        "path": path,
        "bytes": bytes.len(),
        "nodes": nodes,
        "animations": clips,
        "meshes": scene.meshes.len(),
        "materials": scene.materials.len(),
        "images": scene.images.len(),
        "skins": scene.skins.len(),
    }))
}
