mod agent;
mod config;
mod graph;
mod heuristics;
mod rpc;
mod utils;

use anyhow::Result;
use clap::Parser;
use config::Config;
use fnn::rpc::peer::ConnectPeerParams;
use rpc::client::RPCClient;
use std::fs;

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

    log::info!("Using config file: {}", args.config);

    let data = fs::read_to_string(&args.config)?;
    let config: Config = toml::from_str(&data)?;
    let client = RPCClient::new(&config.url);
    log::info!("Found {} external nodes", config.external_nodes.len());
    for addr in config.external_nodes {
        let result = client
            .connect_peer(ConnectPeerParams {
                address: addr.clone(),
                save: Some(true),
            })
            .await;
        match result {
            Ok(_) => {
                log::info!("Connect to external node {addr:?}");
            }
            Err(err) => {
                log::error!("Fail to connect external node {addr:?} {err:?}");
            }
        }
    }
    let agent = agent::Agent::setup(config.agent, client).await?;
    agent.run().await;

    Ok(())
}
