use std::future::Future;

use anyhow::Result;
use ckb_jsonrpc_types::Script;
use fnn::{
    fiber::types::Hash256,
    rpc::{
        channel::{Channel, OpenChannelParams},
        graph::{ChannelInfo, NodeInfo},
        info::NodeInfoResult,
        peer::MultiAddr,
    },
};

/// Query source data
pub trait GraphSource {
    /// Query current fiber node info
    fn node_info(&self) -> impl Future<Output = Result<NodeInfoResult>> + Send;
    /// Query connected channels
    fn local_channels(&self) -> impl Future<Output = Result<Vec<Channel>>> + Send;
    /// Query graph nodes
    fn graph_nodes(&self) -> impl Future<Output = Result<Vec<NodeInfo>>> + Send;
    /// Query graph channels
    fn graph_channels(&self) -> impl Future<Output = Result<Vec<ChannelInfo>>> + Send;
    /// Connect to a peer
    fn connect_peer(&self, addr: MultiAddr) -> impl Future<Output = Result<()>> + Send;
    /// Open a channel
    fn open_channel(
        &self,
        params: OpenChannelParams,
    ) -> impl Future<Output = Result<Hash256>> + Send;
    /// Get Balance of a lock script
    fn get_balance(&self, lock: Script) -> impl Future<Output = Result<u128>> + Send;
}
