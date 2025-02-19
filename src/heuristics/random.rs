use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use anyhow::Result;
use fnn::rpc::peer::PeerId;
use rand::Rng;

use crate::graph::Graph;

pub async fn get_node_scores(
    _graph: Arc<Graph>,
    nodes: HashSet<PeerId>,
) -> Result<HashMap<PeerId, f64>> {
    let mut rng = rand::rng();
    let scores = nodes
        .into_iter()
        .map(|id| (id, rng.random_range(0.0..=1.0)))
        .collect();
    Ok(scores)
}
