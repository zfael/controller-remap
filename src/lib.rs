use anyhow::Result;
use clap::Parser;

pub mod app;
pub mod domain;
#[cfg(windows)]
pub mod platform;

#[derive(Debug, Clone, Parser)]
pub struct Cli {
    #[arg(long, default_value_t = 0)]
    pub slot: u32,
}

pub fn run(cli: Cli) -> Result<()> {
    app::run(cli)
}
