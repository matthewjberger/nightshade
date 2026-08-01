use crate::pre_render::Renderer;
use crate::renderer::lines::{
    GpuNormal, GpuVolume, LINES_PASS, LineFrame, LineVertex, create_pass, hand_over,
};
use crate::renderer::{VulkanRenderer, install_scene_pass, pass_setup};
use nightshade_graphics::queries::lines::{
    gather_bounding_volume_data, gather_lines_data, gather_normal_data,
};
use nightshade_render::data::config::{DebugDraw, RenderBoundingVolume, RenderLine, RenderNormal};
use nightshade_render::queries::projection::matrix_columns;

pub fn install(renderer: &mut VulkanRenderer) {
    let setup = pass_setup(renderer);
    match create_pass(&setup) {
        Ok(pass) => install_scene_pass(renderer, Box::new(pass)),
        Err(error) => tracing::error!("the lines pass was not installed: {error}"),
    }
}

pub fn upload_lines(
    mut renderer: Renderer,
    views: nightshade::ecs::system_param::Res<nightshade_render::data::config::RenderViews>,
    meshes: nightshade::ecs::system_param::Res<nightshade_render::resources::mesh_cache::MeshCache>,
    debug: nightshade::ecs::system_param::Res<DebugDraw>,
    outline: nightshade::ecs::system_param::Res<nightshade_render::resources::outline::Outline>,
    storage: &mut nightshade::ecs::Storage,
) {
    let shading = views
        .active
        .as_ref()
        .map(|camera| camera.effective_shading)
        .unwrap_or(views.fallback_shading);
    let Some(view) = renderer.view.clone() else {
        return;
    };

    let lines = gather_lines_data(storage, &views, &meshes, shading.show_wireframe);
    let volumes = gather_bounding_volume_data(
        storage,
        &views,
        &meshes,
        &debug,
        &outline,
        debug.show_bounding_volumes || shading.show_bounding_volumes,
    );
    let normals = gather_normal_data(
        storage,
        &views,
        &meshes,
        &debug,
        debug.show_normals || shading.show_normals,
    );

    let mut frame = LineFrame {
        view_projection: matrix_columns(&view.view_projection),
        ..LineFrame::default()
    };
    for line in &lines {
        push_line(&mut frame, line);
    }
    frame.volumes = volumes.iter().flatten().map(gpu_volume).collect();
    frame.normals = normals.iter().flatten().map(gpu_normal).collect();

    let Some(pass) = renderer
        .passes
        .iter_mut()
        .find(|(_, pass)| pass.name() == LINES_PASS)
        .map(|(_, pass)| pass.as_mut())
    else {
        return;
    };
    if let Some(pass) = (pass as &mut dyn std::any::Any).downcast_mut() {
        hand_over(pass, frame);
    }
}

fn push_line(frame: &mut LineFrame, line: &RenderLine) {
    let target = if line.depth_mode == 1 {
        &mut frame.on_top
    } else {
        &mut frame.tested
    };
    let color = [line.color.x, line.color.y, line.color.z, line.color.w];
    target.push(LineVertex {
        position: [line.start.x, line.start.y, line.start.z],
        color,
    });
    target.push(LineVertex {
        position: [line.end.x, line.end.y, line.end.z],
        color,
    });
}

fn gpu_volume(volume: &RenderBoundingVolume) -> GpuVolume {
    GpuVolume {
        transform: matrix_columns(&volume.transform),
        center: [volume.center.x, volume.center.y, volume.center.z, 0.0],
        half_extents: [
            volume.half_extents.x,
            volume.half_extents.y,
            volume.half_extents.z,
            0.0,
        ],
        orientation: [
            volume.orientation.x,
            volume.orientation.y,
            volume.orientation.z,
            volume.orientation.w,
        ],
        color: [
            volume.color.x,
            volume.color.y,
            volume.color.z,
            volume.color.w,
        ],
    }
}

fn gpu_normal(normal: &RenderNormal) -> GpuNormal {
    GpuNormal {
        transform: matrix_columns(&normal.transform),
        position: [normal.position.x, normal.position.y, normal.position.z, 1.0],
        direction: [
            normal.normal.x,
            normal.normal.y,
            normal.normal.z,
            normal.length,
        ],
        color: [
            normal.color.x,
            normal.color.y,
            normal.color.z,
            normal.color.w,
        ],
    }
}
