use clap::Parser;
use std::{net::SocketAddr, sync::LazyLock};

#[derive(Parser)]
pub struct AppOptions {
    #[clap(short, long)]
    pub address: Option<SocketAddr>,
}

#[allow(unused)]
pub static ARGS: LazyLock<AppOptions> = LazyLock::new(AppOptions::parse);
