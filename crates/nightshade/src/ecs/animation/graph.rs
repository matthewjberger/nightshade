//! Data-driven animation state machine. States hold a blend tree; transitions
//! fire on parameter conditions and crossfade. The graph is plain data; the free
//! functions below build and interpret it, producing a weighted clip set each
//! frame, the same contract blend trees produce.

use crate::ecs::animation::blend_tree::{BlendTree, collect_blend_tree_clips, evaluate_blend_tree};
use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::visit::EdgeRef;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A state in the graph: a named motion evaluated at a playback speed.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnimationState {
    /// Display name, used by editors and queries.
    pub name: String,
    /// The motion this state plays.
    pub motion: BlendTree,
    /// Playback speed multiplier for this state.
    pub speed: f32,
}

/// A parameter test guarding a transition.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AnimationCondition {
    /// Fires when the parameter is strictly greater than `value`.
    Greater {
        /// Parameter name.
        parameter: String,
        /// Threshold.
        value: f32,
    },
    /// Fires when the parameter is strictly less than `value`.
    Less {
        /// Parameter name.
        parameter: String,
        /// Threshold.
        value: f32,
    },
}

/// A transition between two states: all conditions must hold and the source must
/// have run for `min_state_time` before it fires, then it crossfades over
/// `duration`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnimationTransition {
    /// Conditions that must all be met.
    pub conditions: Vec<AnimationCondition>,
    /// Crossfade duration in seconds; 0 switches instantly.
    pub duration: f32,
    /// Minimum time the source state must run before this transition is eligible.
    pub min_state_time: f32,
}

#[derive(Debug, Clone)]
struct ActiveTransition {
    from: NodeIndex,
    to: NodeIndex,
    elapsed: f32,
    duration: f32,
}

/// A serializable state graph plus its runtime interpreter cursor. The `graph`
/// and `entry` fields serialize; the cursor is runtime-only.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimationGraph {
    graph: DiGraph<AnimationState, AnimationTransition>,
    entry: Option<usize>,
    #[serde(skip)]
    current: Option<NodeIndex>,
    #[serde(skip)]
    state_time: f32,
    #[serde(skip)]
    active_transition: Option<ActiveTransition>,
}

impl Default for AnimationGraph {
    fn default() -> Self {
        Self {
            graph: DiGraph::new(),
            entry: None,
            current: None,
            state_time: 0.0,
            active_transition: None,
        }
    }
}

impl PartialEq for AnimationGraph {
    fn eq(&self, other: &Self) -> bool {
        if self.entry != other.entry
            || self.graph.node_count() != other.graph.node_count()
            || self.graph.edge_count() != other.graph.edge_count()
        {
            return false;
        }
        for (a, b) in self.graph.node_weights().zip(other.graph.node_weights()) {
            if a != b {
                return false;
            }
        }
        let mut self_edges: Vec<_> = self
            .graph
            .edge_references()
            .map(|edge| {
                (
                    edge.source().index(),
                    edge.target().index(),
                    edge.weight().clone(),
                )
            })
            .collect();
        let mut other_edges: Vec<_> = other
            .graph
            .edge_references()
            .map(|edge| {
                (
                    edge.source().index(),
                    edge.target().index(),
                    edge.weight().clone(),
                )
            })
            .collect();
        self_edges.sort_by_key(|edge| (edge.0, edge.1));
        other_edges.sort_by_key(|edge| (edge.0, edge.1));
        self_edges == other_edges
    }
}

/// Adds a state and returns its node index. The first state added becomes the
/// entry state.
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

/// Adds a transition from `from` to `to`.
pub fn animation_graph_add_transition(
    graph: &mut AnimationGraph,
    from: NodeIndex,
    to: NodeIndex,
    transition: AnimationTransition,
) {
    graph.graph.add_edge(from, to, transition);
}

/// Sets the entry state.
pub fn animation_graph_set_entry(graph: &mut AnimationGraph, state: NodeIndex) {
    graph.entry = Some(state.index());
}

/// The names of every state, in insertion order.
pub fn animation_graph_state_names(graph: &AnimationGraph) -> Vec<String> {
    graph
        .graph
        .node_weights()
        .map(|state| state.name.clone())
        .collect()
}

/// The name of the currently active state, if the interpreter has started.
pub fn animation_graph_current_state_name(graph: &AnimationGraph) -> Option<String> {
    graph
        .current
        .and_then(|node| graph.graph.node_weight(node))
        .map(|state| state.name.clone())
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

/// Advances the interpreter by `dt` and writes the weighted clip set into
/// `weights` and the full clip membership into `all_clips`.
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
