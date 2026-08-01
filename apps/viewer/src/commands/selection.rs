use nightshade::prelude::*;
use nightshade_camera::components::{Camera, Projection};
use nightshade_graphics::prelude::Light;
use nightshade_render::components::mesh::RenderMesh;
use nightshade_scene::contract::LocalTransform;
use nightshade_scene::contract::primitives::{Name, Visibility};

use crate::commands::history::push;
use crate::commands::material::read_selected;
use crate::data::{CameraField, LightField};
use crate::queries::selection::{camera_view, euler_to_quat, light_view, quat_to_euler, snapshot};
use crate::resources::{
    CameraView, Edit, LightView, MaterialView, Selection, TransformEdit, TransformSnapshot,
    ViewerResources,
};

pub fn toggle_in_selection(
    stage: &mut crate::resources::Stage<'_>,
    storage: &mut nightshade::ecs::Storage,
    entity: Entity,
) {
    let (primary, held) = {
        let viewer = &mut *stage.viewer;
        (
            viewer.selection.selected,
            viewer.selection.others.contains(&entity),
        )
    };
    match primary {
        None => select_entity(stage, storage, Some(entity)),
        Some(primary) if primary == entity => {
            let others = std::mem::take(&mut stage.viewer.selection.others);
            select_entity(stage, storage, others.first().copied());
            stage.viewer.selection.others = others.into_iter().skip(1).collect();
        }
        Some(_) if held => {
            stage.viewer.selection.others.retain(|held| *held != entity);
        }
        Some(_) => {
            stage.viewer.selection.others.push(entity);
        }
    }
    stage.viewer.view.applied.selected = None;
}

pub fn set_light_field(
    viewer: &mut ViewerResources,
    storage: &mut nightshade::ecs::Storage,
    field: LightField,
) {
    let (Some(entity), Some(before)) = (viewer.selection.selected, viewer.selection.light.clone())
    else {
        return;
    };
    let mut after = before.clone();
    apply_light_field(&mut after, field);
    if let Some(light) = storage.get_mut::<Light>(entity) {
        write_light(light, &after);
    }
    viewer.selection.light = Some(after.clone());
    push(
        &mut viewer.history,
        Edit::Light {
            entity,
            before,
            after,
        },
    );
}

pub fn set_camera_field(
    viewer: &mut ViewerResources,
    storage: &mut nightshade::ecs::Storage,
    field: CameraField,
) {
    let (Some(entity), Some(before)) = (viewer.selection.selected, viewer.selection.camera.clone())
    else {
        return;
    };
    let mut after = before.clone();
    apply_camera_field(&mut after, field);
    if let Some(camera) = storage.get_mut::<Camera>(entity) {
        write_camera(camera, &after);
    }
    viewer.selection.camera = Some(after.clone());
    push(
        &mut viewer.history,
        Edit::Camera {
            entity,
            before,
            after,
        },
    );
}

pub fn select(
    viewer: &mut ViewerResources,
    storage: &nightshade::ecs::Storage,
    entity: Option<Entity>,
) {
    viewer.selection.selected = entity;
    viewer.selection.others.clear();
    viewer.view.inspector_dirty = true;
    viewer.view.detail_dirty = true;

    let Some(entity) = entity else {
        viewer.selection.name.clear();
        viewer.selection.mesh = None;
        viewer.selection.translation = [0.0; 3];
        viewer.selection.rotation = [0.0; 3];
        viewer.selection.rotation_quat = nalgebra_glm::Quat::identity();
        viewer.selection.scale = [1.0; 3];
        viewer.selection.visible = true;
        viewer.selection.material = None;
        viewer.selection.material_values = MaterialView::default();
        viewer.selection.material_loaded = MaterialView::default();
        viewer.selection.light = None;
        viewer.selection.camera = None;
        return;
    };

    viewer.selection.name = storage
        .get::<Name>(entity)
        .map(|name| name.0.clone())
        .filter(|name| !name.is_empty())
        .unwrap_or_else(|| format!("Node {}", entity.id));
    viewer.selection.mesh = storage
        .get::<RenderMesh>(entity)
        .map(|mesh| mesh.name.clone());
    viewer.selection.visible = storage
        .get::<Visibility>(entity)
        .is_none_or(|visibility| visibility.visible);
    if let Some(transform) = storage.get::<LocalTransform>(entity) {
        read_transform(&mut viewer.selection, transform);
    }
    viewer.selection.light = storage.get::<Light>(entity).map(light_view);
    viewer.selection.camera = storage.get::<Camera>(entity).map(camera_view);
}

pub fn read_transform(selection: &mut Selection, transform: &LocalTransform) {
    selection.translation = [
        transform.translation.x,
        transform.translation.y,
        transform.translation.z,
    ];
    selection.rotation = quat_to_euler(transform.rotation);
    selection.rotation_quat = transform.rotation;
    selection.scale = [transform.scale.x, transform.scale.y, transform.scale.z];
}

pub fn apply_transform(viewer: &mut ViewerResources, storage: &mut nightshade::ecs::Storage) {
    let Some(entity) = viewer.selection.selected else {
        return;
    };
    let Some(transform) = storage.get_mut::<LocalTransform>(entity) else {
        return;
    };
    let before = snapshot(transform);
    transform.translation = Vec3::new(
        viewer.selection.translation[0],
        viewer.selection.translation[1],
        viewer.selection.translation[2],
    );
    transform.rotation = euler_to_quat(viewer.selection.rotation);
    transform.scale = Vec3::new(
        viewer.selection.scale[0],
        viewer.selection.scale[1],
        viewer.selection.scale[2],
    );
    viewer.selection.rotation_quat = transform.rotation;
    let after = snapshot(transform);
    if before != after {
        push(
            &mut viewer.history,
            Edit::Transforms(vec![TransformEdit {
                entity,
                before,
                after,
            }]),
        );
    }
}

pub fn restore_transform(
    storage: &mut nightshade::ecs::Storage,
    entity: Entity,
    value: &TransformSnapshot,
) {
    if let Some(transform) = storage.get_mut::<LocalTransform>(entity) {
        transform.translation = value.translation;
        transform.rotation = value.rotation;
        transform.scale = value.scale;
    }
}

fn apply_light_field(view: &mut LightView, field: LightField) {
    match field {
        LightField::Color(value) => view.color = value,
        LightField::Intensity(value) => view.intensity = value,
        LightField::Range(value) => view.range = value,
        LightField::InnerCone(value) => view.inner_cone = value,
        LightField::OuterCone(value) => view.outer_cone = value,
        LightField::CastShadows(value) => view.cast_shadows = value,
    }
}

pub fn write_light(light: &mut Light, view: &LightView) {
    light.color = Vec3::new(view.color[0], view.color[1], view.color[2]);
    light.intensity = view.intensity;
    light.range = view.range;
    light.inner_cone_angle = view.inner_cone.to_radians();
    light.outer_cone_angle = view.outer_cone.to_radians();
    light.cast_shadows = view.cast_shadows;
}

fn apply_camera_field(view: &mut CameraView, field: CameraField) {
    match field {
        CameraField::FieldOfView(value) => view.field_of_view = value,
        CameraField::Near(value) => view.near = value,
        CameraField::Far(value) => view.far = value,
    }
}

pub fn write_camera(camera: &mut Camera, view: &CameraView) {
    match &mut camera.projection {
        Projection::Perspective(perspective) => {
            perspective.y_fov_rad = view.field_of_view.clamp(1.0, 179.0).to_radians();
            perspective.z_near = view.near.max(0.0001);
            perspective.z_far = (view.far > 0.0).then_some(view.far);
        }
        Projection::Orthographic(orthographic) => {
            orthographic.y_mag = view.field_of_view.max(0.0001);
            orthographic.z_near = view.near;
            orthographic.z_far = view.far;
        }
    }
}

pub fn select_entity(
    stage: &mut crate::resources::Stage<'_>,
    storage: &mut nightshade::ecs::Storage,
    entity: Option<Entity>,
) {
    {
        let viewer = &mut *stage.viewer;
        select(viewer, storage, entity);
    }
    let registry = std::mem::take(stage.materials);
    read_selected(stage.viewer, storage, &registry);
    *stage.materials = registry;
}
