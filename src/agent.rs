use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
    time::Duration,
};

use anyhow::{anyhow, bail, Context, Result};
use fnn::{
    fiber::types::{Hash256, Pubkey},
    rpc::{
        channel::{Channel, ListChannelsParams, OpenChannelParams},
        graph::*,
        peer::{ConnectPeerParams, MultiAddr, PeerId},
    },
};

use crate::{
    config::AgentConfig,
    graph::Graph,
    rpc::client::RPCClient,
    utils::{choice_n, get_peer_id_from_addr},
};

#[derive(Debug, Clone)]
struct OpenChannelCmd {
    peer: PeerId,
    funds: u128,
    addresses: Vec<MultiAddr>,
}

/// Autopilot agent
pub struct Agent {
    /// The id of the autopilot node
    pub self_id: Pubkey,
    pub config: AgentConfig,
    pub pending: HashSet<PeerId>,
    pub client: RPCClient,
}

impl Agent {
    pub fn new(self_id: Pubkey, config: AgentConfig, client: RPCClient) -> Self {
        Agent {
            self_id,
            config,
            pending: Default::default(),
            client,
        }
    }

    pub async fn setup(config: AgentConfig, client: RPCClient) -> Result<Self> {
        let node_info = client.node_info().await?;

        log::info!(
            "Node info: {:?} {:?}",
            node_info.node_name,
            PeerId::from_public_key(&node_info.node_id.into())
        );

        let self_id = node_info.node_id;
        Ok(Self::new(self_id, config, client))
    }

    pub async fn run(mut self) {
        log::info!("Start autopilot agent");

        let heuristics_weight = self
            .config
            .heuristics
            .heuristics
            .iter()
            .map(|h| h.weight)
            .sum::<f32>();
        if heuristics_weight != 1.0 {
            log::warn!(
                "Total heuristics weight is {} expected 1.0",
                heuristics_weight
            );
        }

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
            log::trace!(
                "Peer {:?}-{} {}",
                PeerId::from_public_key(&n.node_id.into()),
                n.node_name,
                n.timestamp
            );
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
                "Channel {:?} {} {}",
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
                "Local channel {:?} {} {}",
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
        // check connected pending channels
        for c in local_channels.iter() {
            if self.pending.remove(&c.peer_id) {
                log::info!(
                    "Successfully open channel {:?} {:?} with {:?} funds {}",
                    c.channel_id,
                    c.channel_outpoint,
                    c.peer_id,
                    c.local_balance
                );
            }
        }

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
        // TODO: We should stop open channel and remove peer from pending after timeout
        let num = num.min(self.config.max_pending - self.pending.len());

        let mut ignored: HashSet<PeerId> = local_channels
            .into_iter()
            .map(|c| c.peer_id)
            .chain(self.pending.clone().into_iter())
            .collect();
        ignored.insert(PeerId::from_public_key(&self.self_id.into()));

        let mut nodes: HashSet<PeerId> = HashSet::default();
        let mut addresses: HashMap<PeerId, Vec<MultiAddr>> = HashMap::default();

        for node in graph.nodes() {
            // skip ignored
            let peer = PeerId::from_public_key(&node.node_id.into());
            if ignored.contains(&peer) {
                log::trace!("Skiping node {peer:?}");
                continue;
            }

            // skip unknown addresses
            if node.addresses.is_empty() {
                log::trace!("Skiping node {peer:?} has no known addresses");
                continue;
            }

            if node.auto_accept_min_ckb_funding_amount as u128 > chan_funds {
                log::trace!(
                    "Skiping node {peer:?} require high funding, peer required {} chan_funds {}",
                    node.auto_accept_min_ckb_funding_amount,
                    chan_funds
                );
                continue;
            }

            // store addresses
            addresses
                .entry(peer.clone())
                .or_default()
                .extend(node.addresses.clone());
            nodes.insert(peer);
        }

        let mut scores: Vec<(PeerId, f64)> =
            crate::heuristics::get_node_scores(&self.config.heuristics, graph, nodes)
                .await?
                .into_iter()
                .collect();

        // Insert external nodes scores
        for addr in &self.config.external_nodes {
            let Some(peer) = get_peer_id_from_addr(addr) else {
                log::warn!("Can't find peer id from external address {addr:?}");
                continue;
            };
            if !ignored.contains(&peer) {
                scores.push((peer.clone(), 1.0));
                addresses.insert(peer, vec![addr.to_owned()]);
            }
        }

        log::debug!("Get {} scores", scores.len());
        for (peer, s) in scores.iter() {
            log::trace!("{peer:?} {s}");
        }
        let mut candidates: Vec<OpenChannelCmd> = Vec::default();

        for (peer, _) in choice_n(scores, num) {
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

            let addresses = addresses[&peer].clone();
            let cmd = OpenChannelCmd {
                peer,
                funds: chan_funds,
                addresses,
            };
            candidates.push(cmd);
        }

        log::debug!(
            "Get {} candidates, query num {} pending {}/{}",
            candidates.len(),
            num,
            self.pending.len(),
            self.config.max_pending
        );

        let mut handles = Vec::default();

        // start cmd
        for cmd in candidates {
            let peer = cmd.peer.clone();
            if self.pending.contains(&peer) {
                log::info!("Skipping pending connection {:?}", peer);
                continue;
            }

            self.pending.insert(peer);

            let handle = tokio::spawn(Self::execute(cmd.clone(), self.client.clone()));
            handles.push((cmd, handle));
        }

        // resolve handles
        for (cmd, handle) in handles {
            let OpenChannelCmd {
                peer,
                funds,
                addresses,
            } = cmd;
            match handle.await {
                Ok(Ok(temp_channel_id)) => {
                    log::info!("Initial open channel {temp_channel_id:?} with {peer:?} {addresses:?} funds {funds}");
                    // We must wait for peer to accept the channel
                }
                Ok(Err(err)) => {
                    log::error!("Failed to open channel {peer:?} {addresses:?} {err:?}");
                    self.pending.remove(&peer);
                }
                Err(err) => {
                    log::error!("Failed to execute {peer:?} {addresses:?} {err:?}");
                    self.pending.remove(&peer);
                }
            }
        }

        Ok(())
    }

    async fn execute(cmd: OpenChannelCmd, client: RPCClient) -> Result<Hash256> {
        let OpenChannelCmd {
            peer,
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

        let params = OpenChannelParams {
            peer_id: peer,
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
