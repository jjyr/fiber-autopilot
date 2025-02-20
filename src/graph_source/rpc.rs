use std::future::Future;

use anyhow::Result;
use ckb_jsonrpc_types::Script;
use ckb_sdk::{
    rpc::ckb_indexer::{ScriptType, SearchKey, SearchMode},
    CkbRpcAsyncClient,
};
use fnn::{
    fiber::types::Hash256,
    rpc::{
        channel::{Channel, ListChannelsParams, OpenChannelParams},
        graph::{ChannelInfo, GraphChannelsParams, GraphNodesParams, NodeInfo},
        info::NodeInfoResult,
        peer::{ConnectPeerParams, MultiAddr},
    },
};

use crate::{rpc::client::RPCClient, traits::GraphSource};

#[derive(Clone)]
pub struct RPCGraphSource {
    fiber_client: RPCClient,
    ckb_client: CkbRpcAsyncClient,
}

impl RPCGraphSource {
    pub fn new(fiber_client: RPCClient, ckb_client: CkbRpcAsyncClient) -> Self {
        Self {
            fiber_client,
            ckb_client,
        }
    }
}

#[allow(clippy::manual_async_fn)]
impl GraphSource for RPCGraphSource {
    fn node_info(&self) -> impl Future<Output = Result<NodeInfoResult>> {
        async { self.fiber_client.node_info().await.map_err(Into::into) }
    }

    fn graph_nodes(&self) -> impl Future<Output = Result<Vec<NodeInfo>>> {
        // TODO: fetch all nodes
        async {
            self.fiber_client
                .graph_nodes(GraphNodesParams {
                    limit: None,
                    after: None,
                })
                .await
                .map(|r| r.nodes)
                .map_err(Into::into)
        }
    }

    fn graph_channels(&self) -> impl Future<Output = Result<Vec<ChannelInfo>>> {
        // TODO: fetch all channels
        async {
            self.fiber_client
                .graph_channels(GraphChannelsParams {
                    limit: None,
                    after: None,
                })
                .await
                .map(|r| r.channels)
                .map_err(Into::into)
        }
    }

    fn local_channels(&self) -> impl Future<Output = Result<Vec<Channel>>> {
        async {
            self.fiber_client
                .list_channels(ListChannelsParams {
                    peer_id: None,
                    include_closed: Some(false),
                })
                .await
                .map(|r| r.channels)
                .map_err(Into::into)
        }
    }

    fn connect_peer(&self, addr: MultiAddr) -> impl Future<Output = Result<()>> {
        async {
            self.fiber_client
                .connect_peer(ConnectPeerParams {
                    address: addr,
                    save: Some(true),
                })
                .await
                .map_err(Into::into)
        }
    }

    fn open_channel(&self, params: OpenChannelParams) -> impl Future<Output = Result<Hash256>> {
        async {
            self.fiber_client
                .open_channel(params)
                .await
                .map(|r| r.temporary_channel_id)
                .map_err(Into::into)
        }
    }

    fn get_balance(&self, lock: Script) -> impl Future<Output = Result<u128>> + Send {
        async {
            let search_key = SearchKey {
                script: lock,
                script_type: ScriptType::Lock,
                script_search_mode: Some(SearchMode::Exact),
                filter: None,
                with_data: None,
                group_by_transaction: None,
            };
            let source = self.clone();
            let r = source.ckb_client.get_cells_capacity(search_key).await?;
            let capacity = r.map(|cell| cell.capacity.value()).unwrap_or_default();
            Ok(capacity.into())
        }
    }
}
