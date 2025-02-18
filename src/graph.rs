use std::collections::HashMap;

use fnn::{
    fiber::types::Pubkey,
    rpc::{
        graph::{ChannelInfo, NodeInfo},
        peer::PeerId,
    },
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
    let node_to_idx: HashMap<Pubkey, usize> = nodes
        .iter()
        .enumerate()
        .map(|(index, node)| (node.node_id, index))
        .collect();
    // node index to channel ids map
    let mut node_channels: Vec<Vec<usize>> = vec![Vec::new(); nodes.len()];

    // build node channels
    let mut skip_channel_num = 0;
    'channel: for (c_idx, c) in channels.iter().enumerate() {
        for n in [&c.node1, &c.node2] {
            if !node_to_idx.contains_key(n) {
                let peer = PeerId::from_public_key(&(*n).into());
                log::trace!(
                    "Skiping missing peer {:?} channel {}",
                    peer,
                    &c.channel_outpoint
                );
                skip_channel_num += 1;
                continue 'channel;
            }
        }
        for n in [&c.node1, &c.node2] {
            let n_idx = node_to_idx[n];
            node_channels[n_idx].push(c_idx);
        }
    }
    log::warn!(
        "Total channel {} , valid {} skipped {}",
        channels.len(),
        channels.len() - skip_channel_num,
        skip_channel_num,
    );

    // node index to adjacent nodes map
    let mut edges = vec![Vec::new(); nodes.len()];

    for (n_idx, _node) in nodes.iter().enumerate() {
        // iter node's channels
        for c_idx in &node_channels[n_idx] {
            let c = &channels[*c_idx];

            let n1 = node_to_idx[&c.node1];
            let n2 = node_to_idx[&c.node2];
            // push adjacent node v
            let v_idx = if n_idx == n1 { n2 } else { n1 };
            edges[n_idx].push(v_idx);
        }
    }

    edges
}
