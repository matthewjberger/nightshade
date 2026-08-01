use nalgebra_glm::Vec2;

use std::collections::HashMap;

use crate::contract::blend_tree::{Blend1DChild, Blend2DChild, BlendTree};

pub fn evaluate_blend_tree(
    tree: &BlendTree,
    parameters: &HashMap<String, f32>,
    weight: f32,
    out: &mut Vec<(usize, f32)>,
) {
    if weight <= 0.0 {
        return;
    }
    match tree {
        BlendTree::Clip { clip_index } => {
            out.push((*clip_index, weight));
        }
        BlendTree::Linear1D {
            parameter,
            children,
        } => {
            let value = parameters.get(parameter).copied().unwrap_or(0.0);
            evaluate_linear_1d(children, value, weight, parameters, out);
        }
        BlendTree::Freeform2D {
            parameter_x,
            parameter_y,
            children,
        } => {
            let sample = Vec2::new(
                parameters.get(parameter_x).copied().unwrap_or(0.0),
                parameters.get(parameter_y).copied().unwrap_or(0.0),
            );
            evaluate_freeform_2d(children, sample, weight, parameters, out);
        }
    }
}

pub fn collect_blend_tree_clips(tree: &BlendTree, out: &mut Vec<usize>) {
    match tree {
        BlendTree::Clip { clip_index } => {
            if !out.contains(clip_index) {
                out.push(*clip_index);
            }
        }
        BlendTree::Linear1D { children, .. } => {
            for child in children {
                collect_blend_tree_clips(&child.motion, out);
            }
        }
        BlendTree::Freeform2D { children, .. } => {
            for child in children {
                collect_blend_tree_clips(&child.motion, out);
            }
        }
    }
}

fn evaluate_linear_1d(
    children: &[Blend1DChild],
    value: f32,
    weight: f32,
    parameters: &HashMap<String, f32>,
    out: &mut Vec<(usize, f32)>,
) {
    if children.is_empty() {
        return;
    }
    if value <= children[0].threshold {
        evaluate_blend_tree(&children[0].motion, parameters, weight, out);
        return;
    }
    let last = children.len() - 1;
    if value >= children[last].threshold {
        evaluate_blend_tree(&children[last].motion, parameters, weight, out);
        return;
    }
    for pair in children.windows(2) {
        let low = &pair[0];
        let high = &pair[1];
        if value >= low.threshold && value <= high.threshold {
            let span = high.threshold - low.threshold;
            let ratio = if span > 1.0e-6 {
                (value - low.threshold) / span
            } else {
                0.0
            };
            evaluate_blend_tree(&low.motion, parameters, weight * (1.0 - ratio), out);
            evaluate_blend_tree(&high.motion, parameters, weight * ratio, out);
            return;
        }
    }
}

fn evaluate_freeform_2d(
    children: &[Blend2DChild],
    sample: Vec2,
    weight: f32,
    parameters: &HashMap<String, f32>,
    out: &mut Vec<(usize, f32)>,
) {
    if children.is_empty() {
        return;
    }
    if children.len() == 1 {
        evaluate_blend_tree(&children[0].motion, parameters, weight, out);
        return;
    }
    let mut influences = vec![0.0_f32; children.len()];
    let mut total = 0.0_f32;
    for (index, child) in children.iter().enumerate() {
        let mut influence = 1.0_f32;
        let to_sample = sample - child.position;
        for (other_index, other) in children.iter().enumerate() {
            if other_index == index {
                continue;
            }
            let axis = other.position - child.position;
            let length_squared = axis.dot(&axis);
            if length_squared < 1.0e-8 {
                continue;
            }
            let projection = to_sample.dot(&axis) / length_squared;
            influence = influence.min((1.0 - projection).clamp(0.0, 1.0));
        }
        influences[index] = influence;
        total += influence;
    }
    if total < 1.0e-6 {
        let nearest = children
            .iter()
            .enumerate()
            .min_by(|(_, a), (_, b)| {
                let distance_a = (sample - a.position).magnitude_squared();
                let distance_b = (sample - b.position).magnitude_squared();
                distance_a
                    .partial_cmp(&distance_b)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|(index, _)| index)
            .unwrap_or(0);
        evaluate_blend_tree(&children[nearest].motion, parameters, weight, out);
        return;
    }
    for (index, child) in children.iter().enumerate() {
        let normalized = influences[index] / total;
        if normalized > 0.0 {
            evaluate_blend_tree(&child.motion, parameters, weight * normalized, out);
        }
    }
}
