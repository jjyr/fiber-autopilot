mod agent;
mod centrality;
mod config;
mod graph;
mod rpc;

use agent::Agent;
use anyhow::Result;
use clap::Parser;
use config::Config;
use fnn::rpc::graph::{GraphChannelsParams, GraphNodesParams};
use graph::Graph;
use rpc::client::RPCClient;
use std::{fs, sync::Arc};

/// This is a simple program to demonstrate clap derive usage
#[derive(Parser, Debug)]
#[command(
    version = "1.0",
    about = "Fiber autopilot, automatically open channels with peers"
)]
struct Args {
    /// Sets a custom config file
    #[arg(
        short,
        long,
        value_name = "FILE",
        default_value = "fiber-autopilot.toml"
    )]
    config: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    let args = Args::parse();

    println!("Using config file: {}", args.config);

    let data = fs::read_to_string(&args.config)?;
    let config: Config = toml::from_str(&data)?;

    // TODO: start autopilot
    let client = RPCClient::new(&config.url);

    let node_info = client.node_info().await?;

    println!("Node:");
    println!("{:?}", node_info.node_name);
    println!("{:?}", node_info.node_id);

    println!("Query graph:");

    let nodes = client
        .graph_nodes(GraphNodesParams {
            limit: None,
            after: None,
        })
        .await?;
    println!("Get nodes {}:", nodes.nodes.len());
    for n in &nodes.nodes {
        println!("{:?}-{} {}", n.node_id, n.node_name, n.timestamp);
    }
    let channels = client
        .graph_channels(GraphChannelsParams {
            limit: None,
            after: None,
        })
        .await?;
    println!("Get channels {}:", channels.channels.len());
    for c in &channels.channels {
        println!(
            "{:?}-{} {}",
            c.channel_outpoint, c.capacity, c.created_timestamp
        );
    }

    let graph = Arc::new(Graph::build(nodes.nodes, channels.channels));

    let self_id = node_info.node_id;
    let cfg = agent::Config {
        self_id,
        max_pending: 20,
        min_chan_funds: 10000,
        max_chan_funds: 15000,
    };
    let mut agent = agent::Agent::new(cfg);

    agent.open_channels(42, 20, graph).await?;

    Ok(())
}
