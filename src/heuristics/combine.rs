use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use anyhow::Result;
use fnn::fiber::types::Pubkey;

use crate::{
    config::{Heuristic, HeuristicConfig},
    graph::Graph,
};

pub async fn get_node_scores(
    config: &HeuristicConfig,
    graph: Arc<Graph>,
    nodes: HashSet<Pubkey>,
) -> Result<HashMap<Pubkey, f64>> {
    let mut sub_scores: Vec<HashMap<Pubkey, f64>> = Default::default();
    for h in config.heuristics.iter() {
        let s = match h.heuristic {
            Heuristic::Centrality => {
                super::centrality::get_node_scores(graph.clone(), nodes.clone()).await?
            }
            Heuristic::Random => {
                super::random::get_node_scores(graph.clone(), nodes.clone()).await?
            }
            Heuristic::Richness => {
                super::richness::get_node_scores(graph.clone(), nodes.clone()).await?
            }
        };
        sub_scores.push(s);
    }

    let mut scores: HashMap<Pubkey, f64> = Default::default();
    for n in nodes {
        let mut s = 0.0;
        for (i, h) in config.heuristics.iter().enumerate() {
            s += sub_scores[i][&n] * h.weight as f64;
        }
        scores.insert(n, s);
    }
    Ok(scores)
}
