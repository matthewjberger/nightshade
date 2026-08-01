use crate::contract::graph::AnimationGraph;

pub fn animation_graph_state_names(graph: &AnimationGraph) -> Vec<String> {
    graph
        .graph
        .node_weights()
        .map(|state| state.name.clone())
        .collect()
}

pub fn animation_graph_current_state_name(graph: &AnimationGraph) -> Option<String> {
    graph
        .current
        .and_then(|node| graph.graph.node_weight(node))
        .map(|state| state.name.clone())
}
