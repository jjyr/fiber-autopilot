mod agent;
mod config;
mod graph;
mod graph_source;
mod heuristics;
mod rpc;
mod traits;
mod utils;

use anyhow::Result;
use ckb_sdk::CkbRpcAsyncClient;
use clap::Parser;
use config::Config;
use graph_source::rpc::RPCGraphSource;
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
    let source = {
        let fiber_client = RPCClient::new(&config.fiber.url);
        let ckb_client = CkbRpcAsyncClient::new(&config.ckb.url);
        RPCGraphSource::new(fiber_client, ckb_client)
    };

    let agent = agent::Agent::setup(config.agent, source).await?;
    agent.run().await;

    Ok(())
}
