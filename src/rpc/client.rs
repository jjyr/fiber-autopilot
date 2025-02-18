#![allow(unused)]

use anyhow::Result;
use jsonrpsee::{
    core::{client::ClientT, traits::ToRpcParams},
    http_client::HttpClient,
    rpc_params,
};
use serde::Deserialize;

use fnn::{
    fiber::types::Hash256,
    rpc::{
        cch::{
            GetReceiveBtcOrderParams, ReceiveBTCResponse, ReceiveBtcParams, SendBTCResponse,
            SendBtcParams,
        },
        channel::*,
        dev::{
            AddTlcParams, AddTlcResult, CommitmentSignedParams, RemoveTlcParams,
            SubmitCommitmentTransactionParams, SubmitCommitmentTransactionResult,
        },
        graph::*,
        info::*,
        invoice::{
            InvoiceParams, InvoiceResult, NewInvoiceParams, ParseInvoiceParams, ParseInvoiceResult,
        },
        payment::{GetPaymentCommandParams, GetPaymentCommandResult, SendPaymentCommandParams},
        peer::*,
    },
};

#[derive(Clone)]
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

    // Module Cch
    pub async fn send_btc(&self, params: SendBtcParams) -> Result<SendBTCResponse> {
        self.call("send_btc", rpc_params!(params)).await
    }

    pub async fn receive_btc(&self, params: ReceiveBtcParams) -> Result<ReceiveBTCResponse> {
        self.call("receive_btc", rpc_params!(params)).await
    }

    pub async fn get_receive_btc_order(
        &self,
        params: GetReceiveBtcOrderParams,
    ) -> Result<ReceiveBTCResponse> {
        self.call("get_receive_btc_order", rpc_params!(params))
            .await
    }

    // Module Channel
    pub async fn open_channel(&self, params: OpenChannelParams) -> Result<OpenChannelResult> {
        self.call("open_channel", rpc_params!(params)).await
    }

    pub async fn accept_channel(&self, params: AcceptChannelParams) -> Result<AcceptChannelResult> {
        self.call("accept_channel", rpc_params!(params)).await
    }

    pub async fn list_channels(&self, params: ListChannelsParams) -> Result<ListChannelsResult> {
        self.call("list_channels", rpc_params!(params)).await
    }

    pub async fn shutdown_channel(&self, params: ShutdownChannelParams) -> Result<()> {
        self.call("shutdown_channel", rpc_params!(params)).await
    }

    pub async fn update_channel(&self, params: UpdateChannelParams) -> Result<()> {
        self.call("update_channel", rpc_params!(params)).await
    }

    // Module Dev
    pub async fn commitment_signed(&self, params: CommitmentSignedParams) -> Result<()> {
        self.call("commitment_signed", rpc_params!(params)).await
    }

    pub async fn add_tlc(&self, params: AddTlcParams) -> Result<AddTlcResult> {
        self.call("add_tlc", rpc_params!(params)).await
    }

    pub async fn remove_tlc(&self, params: RemoveTlcParams) -> Result<()> {
        self.call("remove_tlc", rpc_params!(params)).await
    }

    pub async fn submit_commitment_transaction(
        &self,
        params: SubmitCommitmentTransactionParams,
    ) -> Result<SubmitCommitmentTransactionResult> {
        self.call("submit_commitment_transaction", rpc_params!(params))
            .await
    }

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

    // Module Invoice
    pub async fn new_invoice(&self, params: NewInvoiceParams) -> Result<InvoiceResult> {
        self.call("new_invoice", rpc_params!(params)).await
    }

    pub async fn parse_invoice(&self, params: ParseInvoiceParams) -> Result<ParseInvoiceResult> {
        self.call("parse_invoice", rpc_params!(params)).await
    }

    pub async fn get_invoice(&self, params: InvoiceParams) -> Result<InvoiceResult> {
        self.call("get_invoice", rpc_params!(params)).await
    }

    pub async fn cancel_invoice(&self, params: InvoiceParams) -> Result<InvoiceResult> {
        self.call("cancel_invoice", rpc_params!(params)).await
    }

    // Module Payment
    pub async fn send_payment(
        &self,
        params: SendPaymentCommandParams,
    ) -> Result<GetPaymentCommandResult> {
        self.call("send_payment", rpc_params!(params)).await
    }

    pub async fn get_payment(
        &self,
        params: GetPaymentCommandParams,
    ) -> Result<GetPaymentCommandResult> {
        self.call("get_payment", rpc_params!(params)).await
    }

    // Module Peer
    pub async fn connect_peer(&self, params: ConnectPeerParams) -> Result<()> {
        self.call("connect_peer", rpc_params!(params)).await
    }

    pub async fn disconnect_peer(&self, params: DisconnectPeerParams) -> Result<()> {
        self.call("disconnect_peer", rpc_params!(params)).await
    }
}
