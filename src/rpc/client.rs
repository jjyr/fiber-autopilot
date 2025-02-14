use anyhow::{anyhow, Result};
use jsonrpsee::{
    async_client::{Client, ClientBuilder},
    core::{client::ClientT, traits::ToRpcParams},
    http_client::HttpClient,
    rpc_params,
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, error::Error, fmt::Display};

use fnn::{
    fiber::types::Hash256,
    rpc::{cch::*, channel::*, dev::*, graph::*, info::*, invoice::*, payment::*, peer::*},
};

pub struct RPCClient {
    client: HttpClient,
}

impl RPCClient {
    pub fn new(url: &str) -> Self {
        let client = HttpClient::builder().build(url).expect("build client");
        RPCClient { client }
    }

    async fn call<T, R>(&self, method: &str, params: T) -> Result<R>
    where
        T: ToRpcParams + Send,
        R: for<'de> Deserialize<'de>,
    {
        let r = self.client.request(method, params).await?;
        Ok(r)
    }

    // // Module Cch
    // pub async fn send_btc(&self, params: SendBtcParams) -> Result<SendBTCResponse> {
    //     self.call("send_btc", params).await
    // }

    // pub async fn receive_btc(&self, params: ReceiveBtcParams) -> Result<ReceiveBTCResponse> {
    //     self.call("receive_btc", params).await
    // }

    // pub async fn get_receive_btc_order(
    //     &self,
    //     params: GetReceiveBtcOrderParams,
    // ) -> Result<ReceiveBTCResponse> {
    //     self.call("cch_get_receive_btc_order", params).await
    // }

    // // Module Channel
    // pub async fn open_channel(&self, params: OpenChannelParams) -> Result<OpenChannelResult> {
    //     self.call("channel_open_channel", params).await
    // }

    // pub async fn accept_channel(&self, params: AcceptChannelParams) -> Result<AcceptChannelResult> {
    //     self.call("channel_accept_channel", params).await
    // }

    // pub async fn list_channels(&self, params: ListChannelsParams) -> Result<ListChannelsResult> {
    //     self.call("channel_list_channels", params).await
    // }

    // pub async fn shutdown_channel(&self, params: ShutdownChannelParams) -> Result<()> {
    //     self.call("channel_shutdown_channel", params).await
    // }

    // pub async fn update_channel(&self, params: UpdateChannelParams) -> Result<()> {
    //     self.call("channel_update_channel", params).await
    // }

    // // Module Dev
    // pub async fn commitment_signed(&self, channel_id: Hash256) -> Result<()> {
    //     self.call("dev_commitment_signed", (channel_id,)).await
    // }

    // pub async fn add_tlc(&self, params: AddTlcParams) -> Result<AddTlcResult> {
    //     self.call("dev_add_tlc", params).await
    // }

    // pub async fn remove_tlc(&self, params: RemoveTlcParams) -> Result<()> {
    //     self.call("dev_remove_tlc", params).await
    // }

    // pub async fn submit_commitment_transaction(
    //     &self,
    //     params: SubmitCommitmentTransactionParams,
    // ) -> Result<SubmitCommitmentTransactionResult> {
    //     self.call("dev_submit_commitment_transaction", params).await
    // }

    // Module Graph
    pub async fn graph_nodes(&self, params: GraphNodesParams) -> Result<GraphNodesResult> {
        self.call("graph_nodes", rpc_params!(params)).await
    }

    pub async fn graph_channels(&self, params: GraphChannelsParams) -> Result<GraphChannelsResult> {
        self.call("graph_channels", rpc_params!(params)).await
    }

    // Module Info
    pub async fn node_info(&self) -> Result<NodeInfoResult> {
        self.call("node_info", rpc_params!()).await
    }

    // // Module Invoice
    // pub async fn new_invoice(&self, params: NewInvoiceParams) -> Result<InvoiceResult> {
    //     self.call("invoice_new_invoice", params).await
    // }

    // pub async fn parse_invoice(&self, params: ParseInvoiceParams) -> Result<ParseInvoiceResult> {
    //     self.call("invoice_parse_invoice", params).await
    // }

    // pub async fn get_invoice(&self, params: InvoiceParams) -> Result<InvoiceResult> {
    //     self.call("invoice_get_invoice", params).await
    // }

    // pub async fn cancel_invoice(&self, params: InvoiceParams) -> Result<InvoiceResult> {
    //     self.call("invoice_cancel_invoice", params).await
    // }

    // // Module Payment
    // pub async fn send_payment(
    //     &self,
    //     params: SendPaymentCommandParams,
    // ) -> Result<GetPaymentCommandResult> {
    //     self.call("payment_send_payment", params).await
    // }

    // pub async fn get_payment(
    //     &self,
    //     params: GetPaymentCommandParams,
    // ) -> Result<GetPaymentCommandResult> {
    //     self.call("payment_get_payment", params).await
    // }

    // // Module Peer
    // pub async fn connect_peer(&self, params: ConnectPeerParams) -> Result<()> {
    //     self.call("peer_connect_peer", params).await
    // }

    // pub async fn disconnect_peer(&self, params: DisconnectPeerParams) -> Result<()> {
    //     self.call("peer_disconnect_peer", params).await
    // }
}
