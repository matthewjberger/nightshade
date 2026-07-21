//! Parametric blend trees. A tree maps named parameters to a set of weighted
//! clips, which the pose pipeline blends into the final pose. The tree is plain
//! data; the free functions below interpret it.

use nalgebra_glm::Vec2;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A node in a blend tree: a leaf clip, a 1D parameter blend, or a 2D freeform
/// blend. Children are themselves trees, so blends nest.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BlendTree {
    /// A single clip at full weight.
    Clip {
        /// Index into the player's clip list.
        clip_index: usize,
    },
    /// Blends children along one parameter, each child anchored at a threshold.
    Linear1D {
        /// Parameter name read from the player's parameter map.
        parameter: String,
        /// Children ordered by ascending threshold.
        children: Vec<Blend1DChild>,
    },
    /// Blends children across a 2D parameter plane by gradient bands.
    Freeform2D {
        /// Parameter driving the horizontal axis.
        parameter_x: String,
        /// Parameter driving the vertical axis.
        parameter_y: String,
        /// Children placed at 2D sample positions.
        children: Vec<Blend2DChild>,
    },
}

/// A child of a [`BlendTree::Linear1D`] anchored at a parameter threshold.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Blend1DChild {
    /// Parameter value at which this child reaches full influence.
    pub threshold: f32,
    /// The child motion.
    pub motion: BlendTree,
}

/// A child of a [`BlendTree::Freeform2D`] placed at a 2D sample position.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Blend2DChild {
    /// Sample position in the parameter plane.
    pub position: Vec2,
    /// The child motion.
    pub motion: BlendTree,
}

/// Accumulates `(clip_index, weight)` contributions into `out`, scaling the whole
/// subtree by `weight`. Missing parameters read as 0.0.
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

/// Collects every clip index this tree can reference, in stable order,
/// deduplicated.
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
