use std::collections::HashMap;

use petgraph::graph::NodeIndex;
use petgraph::visit::EdgeRef;

use crate::commands::blend_tree::{collect_blend_tree_clips, evaluate_blend_tree};
use crate::contract::blend_tree::BlendTree;
use crate::contract::graph::{
    ActiveTransition, AnimationCondition, AnimationGraph, AnimationState, AnimationTransition,
};

pub fn animation_graph_add_state(
    graph: &mut AnimationGraph,
    name: impl Into<String>,
    motion: BlendTree,
) -> NodeIndex {
    let node = graph.graph.add_node(AnimationState {
        name: name.into(),
        motion,
        speed: 1.0,
    });
    if graph.entry.is_none() {
        graph.entry = Some(node.index());
    }
    node
}

pub fn animation_graph_add_transition(
    graph: &mut AnimationGraph,
    from: NodeIndex,
    to: NodeIndex,
    transition: AnimationTransition,
) {
    graph.graph.add_edge(from, to, transition);
}

pub fn animation_graph_set_entry(graph: &mut AnimationGraph, state: NodeIndex) {
    graph.entry = Some(state.index());
}

pub fn evaluate_animation_graph(
    graph: &mut AnimationGraph,
    parameters: &HashMap<String, f32>,
    dt: f32,
    weights: &mut Vec<(usize, f32)>,
    all_clips: &mut Vec<usize>,
) {
    if graph.current.is_none() {
        graph.current = graph.entry.map(NodeIndex::new);
        graph.state_time = 0.0;
    }
    let Some(current) = graph.current else {
        return;
    };
    let Some(state) = graph.graph.node_weight(current) else {
        return;
    };
    graph.state_time += dt * state.speed;

    if graph.active_transition.is_none() {
        for (target, transition) in graph_out_transitions(graph, current) {
            if transition_ready(&transition, parameters, graph.state_time) {
                if transition.duration <= 0.0 {
                    graph.current = Some(target);
                    graph.state_time = 0.0;
                } else {
                    graph.active_transition = Some(ActiveTransition {
                        from: current,
                        to: target,
                        elapsed: 0.0,
                        duration: transition.duration,
                    });
                }
                break;
            }
        }
    }

    if let Some(mut transition) = graph.active_transition.take() {
        transition.elapsed += dt;
        let progress = (transition.elapsed / transition.duration.max(1.0e-4)).clamp(0.0, 1.0);
        if let Some(from_state) = graph.graph.node_weight(transition.from) {
            let motion = from_state.motion.clone();
            collect_blend_tree_clips(&motion, all_clips);
            evaluate_blend_tree(&motion, parameters, 1.0 - progress, weights);
        }
        if let Some(to_state) = graph.graph.node_weight(transition.to) {
            let motion = to_state.motion.clone();
            collect_blend_tree_clips(&motion, all_clips);
            evaluate_blend_tree(&motion, parameters, progress, weights);
        }
        if transition.elapsed >= transition.duration {
            graph.current = Some(transition.to);
            graph.state_time = 0.0;
            graph.active_transition = None;
        } else {
            graph.active_transition = Some(transition);
        }
        return;
    }

    if let Some(state) = graph.graph.node_weight(current) {
        let motion = state.motion.clone();
        collect_blend_tree_clips(&motion, all_clips);
        evaluate_blend_tree(&motion, parameters, 1.0, weights);
    }
}

fn condition_met(condition: &AnimationCondition, parameters: &HashMap<String, f32>) -> bool {
    match condition {
        AnimationCondition::Greater { parameter, value } => {
            parameters.get(parameter).copied().unwrap_or(0.0) > *value
        }
        AnimationCondition::Less { parameter, value } => {
            parameters.get(parameter).copied().unwrap_or(0.0) < *value
        }
    }
}

fn transition_ready(
    transition: &AnimationTransition,
    parameters: &HashMap<String, f32>,
    state_time: f32,
) -> bool {
    state_time >= transition.min_state_time
        && transition
            .conditions
            .iter()
            .all(|condition| condition_met(condition, parameters))
}

fn graph_out_transitions(
    graph: &AnimationGraph,
    node: NodeIndex,
) -> Vec<(NodeIndex, AnimationTransition)> {
    let mut edges: Vec<_> = graph
        .graph
        .edges(node)
        .map(|edge| (edge.target(), edge.weight().clone()))
        .collect();
    edges.reverse();
    edges
}
