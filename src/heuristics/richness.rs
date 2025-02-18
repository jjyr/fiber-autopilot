use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use anyhow::Result;
use fnn::fiber::types::Pubkey;

use crate::graph::Graph;

/// Determine the minimum size that channel count as positive
const MIN_MEDIAN_CHAN_CAP_FRACTION: u128 = 4;

pub async fn get_node_scores(
    graph: Arc<Graph>,
    nodes: HashSet<Pubkey>,
) -> Result<HashMap<Pubkey, f64>> {
    // get median
    let mut chan_caps = Vec::default();
    for c in graph.channels() {
        chan_caps.push(c.capacity);
    }
    chan_caps.sort_unstable();
    let median_cap = chan_caps
        .get(chan_caps.len() / 2)
        .cloned()
        .unwrap_or_default();

    // count the number of largest channels for each node
    let mut node_chan_num: HashMap<Pubkey, i32> = HashMap::default();
    let mut count_chan = |n: Pubkey, neg: bool| {
        if nodes.contains(&n) {
            if neg {
                *node_chan_num.entry(n).or_default() -= 1;
            } else {
                *node_chan_num.entry(n).or_default() += 1;
            }
        }
    };

    for c in graph.channels() {
        let neg = c.capacity < median_cap / MIN_MEDIAN_CHAN_CAP_FRACTION;
        count_chan(c.node1, neg);
        count_chan(c.node2, neg);
    }

    let max_chan_num = node_chan_num.values().max().cloned().unwrap_or_default();

    let scores = nodes
        .into_iter()
        .map(|id| {
            let chan_num = node_chan_num.get(&id).cloned().unwrap_or_default();

            let s = if chan_num > 0 {
                chan_num as f64 / max_chan_num as f64
            } else {
                0.0
            };

            (id, s)
        })
        .collect();
    Ok(scores)
}
