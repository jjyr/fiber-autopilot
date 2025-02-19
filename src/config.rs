use fnn::{fiber::serde_utils::U128Hex, rpc::peer::MultiAddr};
use serde::{Deserialize, Serialize};
use serde_with::serde_as;

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub url: String,
    pub agent: AgentConfig,
}

#[derive(Serialize, Deserialize)]
pub enum Heuristic {
    Random,
    Centrality,
    Richness,
}

#[derive(Serialize, Deserialize)]
pub struct HeuristicItem {
    pub heuristic: Heuristic,
    pub weight: f32,
}

#[derive(Serialize, Deserialize)]
pub struct HeuristicConfig {
    pub heuristics: Vec<HeuristicItem>,
}

impl Default for HeuristicConfig {
    fn default() -> Self {
        Self {
            heuristics: vec![HeuristicItem {
                heuristic: Heuristic::Centrality,
                weight: 1.0,
            }],
        }
    }
}

#[serde_as]
#[derive(Serialize, Deserialize)]
pub struct AgentConfig {
    /// Open channals to external nodes without scoring
    pub external_nodes: Vec<MultiAddr>,
    /// TODO: Fiber should provide a query RPC
    /// Available Funds
    #[serde_as(as = "U128Hex")]
    pub available_funds: u128,
    /// Max channels
    pub max_chan_num: usize,
    /// Interval seconds
    pub interval: u64,
    /// Max pending channels
    pub max_pending: usize,
    /// Minimal chan size
    #[serde_as(as = "U128Hex")]
    pub min_chan_funds: u128,
    /// Max chan size
    #[serde_as(as = "U128Hex")]
    pub max_chan_funds: u128,
    #[serde(default, flatten)]
    pub heuristics: HeuristicConfig,
}
