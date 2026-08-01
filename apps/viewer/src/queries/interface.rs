use nightshade_ui::data::icons::IconGlyph;

use crate::data::{AXES, RowKind};
use crate::resources::{Gizmo, ModelStats};

pub fn gizmo_index(gizmo: &Gizmo) -> Option<usize> {
    use crate::data::GizmoMode;
    gizmo.enabled.then_some(match gizmo.mode {
        GizmoMode::Move => 0,
        GizmoMode::Rotate => 1,
        GizmoMode::Scale => 2,
    })
}

pub fn stat_strings(stats: Option<&ModelStats>) -> [String; 6] {
    let Some(stats) = stats else {
        return std::array::from_fn(|_| "-".to_string());
    };
    [
        group_digits(stats.meshes),
        group_digits(stats.vertices),
        group_digits(stats.triangles),
        group_digits(stats.materials),
        group_digits(stats.textures),
        format!(
            "{:.2} x {:.2} x {:.2}",
            stats.dimensions[0], stats.dimensions[1], stats.dimensions[2]
        ),
    ]
}

fn group_digits(value: u32) -> String {
    let digits = value.to_string();
    let bytes = digits.as_bytes();
    let mut grouped = String::new();
    for (index, byte) in bytes.iter().enumerate() {
        if index > 0 && (bytes.len() - index).is_multiple_of(3) {
            grouped.push(',');
        }
        grouped.push(*byte as char);
    }
    grouped
}

pub fn icon_of(kind: RowKind) -> IconGlyph {
    use nightshade_ui::data::icons::material;
    match kind {
        RowKind::Mesh => material::VIEW_IN_AR,
        RowKind::Light => material::LIGHTBULB,
        RowKind::Camera => material::VIDEOCAM,
        RowKind::Node => material::FOLDER,
    }
}

pub fn axis_of(command_id: u32) -> Option<[f32; 3]> {
    AXES.get(command_id.checked_sub(1)? as usize)
        .map(|(axis, ..)| *axis)
}
