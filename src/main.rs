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
use tokio::task::JoinSet;
use tracing::{error, info};

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

fn init_log() {
    tracing_subscriber::fmt().init();
}

#[tokio::main]
async fn main() -> Result<()> {
    init_log();

    let args = Args::parse();
    info!("Using config file: {}", args.config);

    let data = fs::read_to_string(&args.config)?;
    let config: Config = toml::from_str(&data)?;
    let source = {
        let fiber_client = RPCClient::new(&config.fiber.url);
        let ckb_client = CkbRpcAsyncClient::new(&config.ckb.url);
        RPCGraphSource::new(fiber_client, ckb_client)
    };

    let handle: JoinSet<_> = config
        .agents
        .into_iter()
        .enumerate()
        .map(|(index, config)| {
            let name = format!("agent-{index}");
            let source = source.clone();
            tokio::spawn(async {
                let token = config.token.name().to_string();
                match agent::Agent::setup(name, config, source).await {
                    Ok(agent) => {
                        agent.run().await;
                    }
                    Err(err) => {
                        error!("Failed to setup agent {token} error {err:?}");
                    }
                }
            })
        })
        .collect();

    info!("All agents are started {}", handle.len());

    for r in handle.join_all().await {
        if let Err(err) = r {
            error!("join error {err:?}");
        }
    }

    Ok(())
}
