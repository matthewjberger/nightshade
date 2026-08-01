use crate::data::Holding;
use nalgebra_glm::{Vec2, Vec4, vec4};
use nightshade::ecs::Entity;

#[derive(Clone, Debug)]
pub struct NodeGraphSettings {
    pub node_width: f32,
    pub header_height: f32,
    pub row_height: f32,
    pub node_tail: f32,
    pub note_height: f32,
    pub corner: f32,
    pub edge_width: f32,
    pub socket_radius: f32,
    pub title_size: f32,
    pub title_inset: Vec2,
    pub label_size: f32,
    pub label_inset: f32,
    pub label_advance: f32,
    pub grip_inset: f32,
    pub wire_reach: f32,
    pub wire_width: f32,
    pub grab_slack: f32,
    pub grid_spacing: f32,
    pub grid_width: f32,
    pub body_tint: Vec4,
    pub header_tint: Vec4,
    pub edge_tint: Vec4,
    pub held_tint: Vec4,
    pub socket_tint: Vec4,
    pub lit_tint: Vec4,
    pub wire_tint: Vec4,
    pub dangling_tint: Vec4,
    pub grid_tint: Vec4,
    pub title_tint: Vec4,
    pub label_tint: Vec4,
    pub note_tint: Vec4,
    pub note_field_tint: Vec4,
    pub note_lit_tint: Vec4,
}

impl Default for NodeGraphSettings {
    fn default() -> Self {
        Self {
            node_width: 184.0,
            header_height: 26.0,
            row_height: 20.0,
            node_tail: 8.0,
            note_height: 20.0,
            corner: 6.0,
            edge_width: 1.5,
            socket_radius: 6.0,
            title_size: 14.0,
            title_inset: Vec2::new(10.0, 6.0),
            label_size: 11.0,
            label_inset: 12.0,
            label_advance: 0.52,
            grip_inset: 5.0,
            wire_reach: 60.0,
            wire_width: 2.5,
            grab_slack: 8.0,
            grid_spacing: 32.0,
            grid_width: 1.0,
            body_tint: vec4(0.16, 0.17, 0.21, 0.96),
            header_tint: vec4(0.24, 0.26, 0.32, 1.0),
            edge_tint: vec4(0.06, 0.06, 0.08, 1.0),
            held_tint: vec4(0.96, 0.82, 0.36, 1.0),
            socket_tint: vec4(0.62, 0.68, 0.78, 1.0),
            lit_tint: vec4(0.98, 0.92, 0.58, 1.0),
            wire_tint: vec4(0.72, 0.80, 0.92, 0.92),
            dangling_tint: vec4(0.96, 0.82, 0.36, 0.92),
            grid_tint: vec4(1.0, 1.0, 1.0, 0.05),
            title_tint: vec4(0.92, 0.94, 0.98, 1.0),
            label_tint: vec4(0.74, 0.78, 0.86, 1.0),
            note_tint: vec4(0.66, 0.80, 0.70, 1.0),
            note_field_tint: vec4(0.20, 0.22, 0.28, 1.0),
            note_lit_tint: vec4(0.30, 0.33, 0.42, 1.0),
        }
    }
}

#[derive(Default)]
pub struct GraphView {
    pub canvas: Option<Entity>,
    pub pan: Vec2,
}

#[derive(Default)]
pub struct GraphGrip {
    pub pointer: Vec2,
    pub local: Vec2,
    pub step: Vec2,
    pub tracking: bool,
    pub inside: bool,
    pub holding: Holding,
    pub over: Holding,
    pub letting_go: bool,
    pub cutting: bool,
}
