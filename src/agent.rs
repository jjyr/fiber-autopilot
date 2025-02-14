use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use anyhow::{bail, Result};
use fnn::{
    fiber::types::Pubkey,
    rpc::{graph::NodeInfo, peer::MultiAddr},
};
use rand::distr::{weighted::WeightedIndex, Distribution};

use crate::{centrality::get_node_scores, graph::Graph};

pub struct Config {
    // The id of the autopilot node
    pub self_id: Pubkey,
    // Max pending channels
    pub max_pending: usize,
    // Minimal chan size
    pub min_chan_funds: u64,
    // Max chan size
    pub max_chan_funds: u64,
}

// Autopilot agent
pub struct Agent {
    pub config: Config,
    pub state: Vec<Pubkey>,
    pub pending: HashSet<Pubkey>,
    pub connected_nodes: Vec<Pubkey>,
}

#[derive(Debug)]
struct OpenChannelCmd {
    node_id: Pubkey,
    funds: u64,
    addresses: Vec<MultiAddr>,
}

impl Agent {
    pub fn new(config: Config) -> Self {
        Agent {
            config,
            state: Default::default(),
            pending: Default::default(),
            connected_nodes: Default::default(),
        }
    }

    pub async fn open_channels(
        &mut self,
        mut available_funds: u64,
        num: usize,
        graph: Arc<Graph>,
    ) -> Result<()> {
        let chan_funds = self.config.max_chan_funds.min(available_funds);
        if chan_funds < self.config.min_chan_funds {
            bail!(
                "Not enough funds to open channel, available {} required {}",
                available_funds,
                self.config.min_chan_funds
            );
        }

        let mut ignored: HashSet<Pubkey> = self
            .connected_nodes
            .clone()
            .into_iter()
            .chain(self.pending.clone())
            .collect();
        ignored.insert(self.config.self_id);

        let mut nodes: HashSet<Pubkey> = HashSet::default();
        let mut addresses: HashMap<Pubkey, Vec<MultiAddr>> = HashMap::default();

        for node in graph.nodes() {
            // skip ignored
            if ignored.contains(&node.node_id) {
                log::trace!("Skiping node {:?}", node.node_id);
                continue;
            }

            // skip unknown addresses
            if node.addresses.is_empty() {
                log::trace!("Skiping node {:?} has no known addresses", node.node_id);
                continue;
            }

            // store addresses
            addresses
                .entry(node.node_id)
                .or_default()
                .extend(node.addresses.clone());
            nodes.insert(node.node_id);
        }

        let scores: Vec<(Pubkey, f64)> = get_node_scores(graph, nodes).await?.into_iter().collect();
        let mut rng = rand::rng();
        let dist = WeightedIndex::new(scores.iter().map(|item| item.1)).unwrap();
        let mut candidates: Vec<OpenChannelCmd> = Vec::default();
        for index in dist.sample_iter(&mut rng).take(num) {
            let node_id = scores[index].0;
            let chan_funds = available_funds.min(chan_funds);
            available_funds -= chan_funds;

            if chan_funds < self.config.min_chan_funds {
                log::trace!(
                    "Chan funds too small chan_funds {} required {}",
                    chan_funds,
                    self.config.min_chan_funds
                );
                break;
            }

            let cmd = OpenChannelCmd {
                node_id,
                funds: chan_funds,
                addresses: addresses[&node_id].clone(),
            };
            candidates.push(cmd);
        }

        if candidates.is_empty() {
            log::info!("No candidates");
        }

        if self.pending.len() >= self.config.max_pending {
            log::debug!(
                "Stop open connections since we had too many pending channels {} max_pending {}",
                self.pending.len(),
                self.config.max_pending
            );
            return Ok(());
        }

        for cmd in candidates {
            // TODO:connect peer
            log::debug!("Connect {cmd:?}")
        }

        Ok(())
    }
}
