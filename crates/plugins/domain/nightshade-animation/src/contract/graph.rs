use crate::contract::blend_tree::BlendTree;
use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::visit::EdgeRef;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnimationState {
    pub name: String,
    pub motion: BlendTree,
    pub speed: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AnimationCondition {
    Greater { parameter: String, value: f32 },
    Less { parameter: String, value: f32 },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnimationTransition {
    pub conditions: Vec<AnimationCondition>,
    pub duration: f32,
    pub min_state_time: f32,
}

#[derive(Debug, Clone)]
pub struct ActiveTransition {
    pub from: NodeIndex,
    pub to: NodeIndex,
    pub elapsed: f32,
    pub duration: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimationGraph {
    pub graph: DiGraph<AnimationState, AnimationTransition>,
    pub entry: Option<usize>,
    #[serde(skip)]
    pub current: Option<NodeIndex>,
    #[serde(skip)]
    pub state_time: f32,
    #[serde(skip)]
    pub active_transition: Option<ActiveTransition>,
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
