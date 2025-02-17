use serde::{Deserialize, Serialize};

use crate::agent;

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub url: String,
    pub agent: agent::Config,
}
