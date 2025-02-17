use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
    time::Duration,
};

use anyhow::{anyhow, bail, Context, Result};
use fnn::{
    fiber::{
        serde_utils::U128Hex,
        types::{Hash256, Pubkey},
    },
    rpc::{
        channel::{Channel, ListChannelsParams, OpenChannelParams},
        graph::*,
        peer::{ConnectPeerParams, MultiAddr, PeerId},
    },
};
use serde::*;
use serde_with::serde_as;

use crate::{centrality::get_node_scores, graph::Graph, rpc::client::RPCClient, utils::choice_n};

#[derive(Debug, Clone)]
struct OpenChannelCmd {
    node_id: Pubkey,
    funds: u128,
    addresses: Vec<MultiAddr>,
}

#[serde_as]
#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
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
}

/// Autopilot agent
pub struct Agent {
    /// The id of the autopilot node
    pub self_id: Pubkey,
    pub config: Config,
    pub pending: HashSet<Pubkey>,
    pub client: RPCClient,
}

impl Agent {
    pub fn new(self_id: Pubkey, config: Config, client: RPCClient) -> Self {
        Agent {
            self_id,
            config,
            pending: Default::default(),
            client,
        }
    }

    pub async fn setup(config: Config, client: RPCClient) -> Result<Self> {
        let node_info = client.node_info().await?;

        log::info!(
            "Node info: {:?} {:?}",
            node_info.node_name,
            node_info.node_id
        );

        let self_id = node_info.node_id;
        Ok(Self::new(self_id, config, client))
    }

    pub async fn run(mut self) {
        log::info!("Start autopilot agent");
        loop {
            if let Err(err) = self.run_once().await {
                log::error!("Run once {err:?}");
            }
            let interval = Duration::from_secs(self.config.interval);
            tokio::time::sleep(interval).await;
        }
    }

    pub async fn run_once(&mut self) -> Result<()> {
        let nodes = self
            .client
            .graph_nodes(GraphNodesParams {
                limit: None,
                after: None,
            })
            .await?;
        for n in &nodes.nodes {
            log::trace!("peer {:?}-{} {}", n.node_id, n.node_name, n.timestamp);
        }
        let channels = self
            .client
            .graph_channels(GraphChannelsParams {
                limit: None,
                after: None,
            })
            .await?;
        for c in &channels.channels {
            log::trace!(
                "channel {:?}-{} {}",
                c.channel_outpoint,
                c.capacity,
                c.created_timestamp
            );
        }

        let local_channels = self
            .client
            .list_channels(ListChannelsParams {
                peer_id: None,
                include_closed: Some(false),
            })
            .await?;

        for c in &local_channels.channels {
            log::trace!(
                "local channel {:?}-{} {}",
                c.channel_outpoint,
                c.peer_id,
                c.channel_id
            );
        }

        log::info!(
            "Query {} nodes {} channels {} locals from the network",
            nodes.nodes.len(),
            channels.channels.len(),
            local_channels.channels.len()
        );
        let graph = Arc::new(Graph::build(nodes.nodes, channels.channels));

        let available_funds = self.config.available_funds;
        let num = (self
            .config
            .max_chan_num
            .saturating_sub(local_channels.channels.len()))
        .min(20);
        self.open_channels(available_funds, num, graph, local_channels.channels)
            .await
    }

    pub(crate) async fn open_channels(
        &mut self,
        mut available_funds: u128,
        num: usize,
        graph: Arc<Graph>,
        local_channels: Vec<Channel>,
    ) -> Result<()> {
        let chan_funds = self.config.max_chan_funds.min(available_funds);
        if chan_funds < self.config.min_chan_funds {
            bail!(
                "Not enough funds to open channel, available {} required {}",
                available_funds,
                self.config.min_chan_funds
            );
        }

        if self.pending.len() >= self.config.max_pending {
            log::debug!(
                "Stop open connections since we had too many pending channels {} max_pending {}",
                self.pending.len(),
                self.config.max_pending
            );
            return Ok(());
        }

        // open channels up to max_pending
        let num = num.min(self.config.max_pending - self.pending.len());

        let mut ignored: HashSet<PeerId> = local_channels
            .into_iter()
            .map(|c| c.peer_id)
            .chain(
                self.pending
                    .clone()
                    .into_iter()
                    .map(|pk| PeerId::from_public_key(&pk.into())),
            )
            .collect();
        ignored.insert(PeerId::from_public_key(&self.self_id.into()));

        let mut nodes: HashSet<Pubkey> = HashSet::default();
        let mut addresses: HashMap<Pubkey, Vec<MultiAddr>> = HashMap::default();

        for node in graph.nodes() {
            // skip ignored
            if ignored.contains(&PeerId::from_public_key(&node.node_id.into())) {
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
        log::debug!("Get {} scores", scores.len());
        let mut candidates: Vec<OpenChannelCmd> = Vec::default();

        for (node_id, _) in choice_n(scores, num) {
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

        log::debug!("Get {} candidates, query num {}", candidates.len(), num);

        let mut handles = Vec::default();

        // start cmd
        for cmd in candidates {
            if self.pending.contains(&cmd.node_id) {
                log::info!("Skipping pending connection {:?}", cmd.node_id);
                continue;
            }

            self.pending.insert(cmd.node_id);

            let handle = tokio::spawn(Self::execute(cmd.clone(), self.client.clone()));
            handles.push((cmd, handle));
        }

        // TODO: Try to remove the blocking here, we should be able to handle new open_channels
        // resolve handles
        for (cmd, handle) in handles {
            let OpenChannelCmd {
                node_id,
                funds,
                addresses,
            } = cmd;
            let peer = PeerId::from_public_key(&node_id.into());
            match handle.await {
                Ok(Ok(temp_channel_id)) => {
                    log::info!("Open channel {temp_channel_id:?} with {peer:?} {addresses:?} funds {funds}");
                    self.pending.remove(&cmd.node_id);
                }
                Ok(Err(err)) => {
                    log::error!("Failed to open channel {peer:?} {addresses:?} {err:?}");
                    self.pending.remove(&cmd.node_id);
                }
                Err(err) => {
                    log::error!("Failed to execute {peer:?} {addresses:?} {err:?}");
                    self.pending.remove(&cmd.node_id);
                }
            }
        }

        let local_channels = self
            .client
            .list_channels(ListChannelsParams {
                peer_id: None,
                include_closed: Some(true),
            })
            .await?;

        for c in &local_channels.channels {
            log::trace!(
                "local channel {:?}-{} {}",
                c.channel_outpoint,
                c.peer_id,
                c.channel_id
            );
        }

        Ok(())
    }

    async fn execute(cmd: OpenChannelCmd, client: RPCClient) -> Result<Hash256> {
        let OpenChannelCmd {
            node_id,
            funds,
            addresses,
        } = cmd;

        let address = addresses
            .first()
            .cloned()
            .ok_or_else(|| anyhow!("No address"))?;

        let params = ConnectPeerParams {
            address,
            save: Some(true),
        };
        client.connect_peer(params).await.context("connect peer")?;

        // wait
        tokio::time::sleep(Duration::from_secs(3)).await;

        let peer_id = PeerId::from_public_key(&node_id.into());
        let params = OpenChannelParams {
            peer_id,
            funding_amount: funds,
            commitment_fee_rate: None,
            public: None,
            funding_fee_rate: None,
            commitment_delay_epoch: None,
            funding_udt_type_script: None,
            shutdown_script: None,
            max_tlc_value_in_flight: None,
            max_tlc_number_in_flight: None,
            tlc_expiry_delta: None,
            tlc_fee_proportional_millionths: None,
            tlc_min_value: None,
        };
        let r = client.open_channel(params).await.context("open channel")?;
        Ok(r.temporary_channel_id)
    }
}
