use std::collections::HashMap;

use fnn::{
    fiber::types::Pubkey,
    rpc::graph::{ChannelInfo, NodeInfo},
};

pub struct Graph {
    nodes: Vec<NodeInfo>,
    channels: Vec<ChannelInfo>,
    edges: Vec<Vec<usize>>,
}

impl Graph {
    pub fn build(nodes: Vec<NodeInfo>, channels: Vec<ChannelInfo>) -> Self {
        let edges = compute_edges(&nodes, &channels);
        Self {
            nodes,
            channels,
            edges,
        }
    }

    pub fn nodes(&self) -> &[NodeInfo] {
        &self.nodes
    }

    pub fn channels(&self) -> &[ChannelInfo] {
        &self.channels
    }

    pub fn edges(&self) -> &[Vec<usize>] {
        &self.edges
    }
}

/// Compuate adjacent nodes
fn compute_edges(nodes: &[NodeInfo], channels: &[ChannelInfo]) -> Vec<Vec<usize>> {
    // node pubkey to index map
    let node_key_to_index: HashMap<Pubkey, usize> = nodes
        .iter()
        .enumerate()
        .map(|(index, node)| (node.node_id, index))
        .collect();
    // node index to channel ids map
    let mut node_channels: Vec<Vec<usize>> = vec![Vec::new(); nodes.len()];

    // build node channels
    for (c_idx, c) in channels.iter().enumerate() {
        for n in [&c.node1, &c.node2] {
            let n_idx = node_key_to_index[n];
            node_channels[n_idx].push(c_idx);
        }
    }

    // node index to adjacent nodes map
    let mut edges = vec![Vec::new(); nodes.len()];

    for (n_idx, _node) in nodes.iter().enumerate() {
        // iter node's channels
        for c_idx in &node_channels[n_idx] {
            let c = &channels[*c_idx];
            let n1 = node_key_to_index[&c.node1];
            let n2 = node_key_to_index[&c.node2];
            // push adjacent node v
            let v_idx = if n_idx == n1 { n2 } else { n1 };
            edges[n_idx].push(v_idx);
        }
    }

    edges
}
