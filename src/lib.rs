use std::io;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use clap::Parser;
use russh::server;
use russh::server::Config;
use russh_keys::key::KeyPair;
use tracing::metadata::LevelFilter;
use tracing::{info, subscriber};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::{fmt, Registry};

use crate::defeat_record::MahouSyouzyoRecord;
use crate::hiiragi_utena::HiiragiUtena;

mod defeat_record;
mod hiiragi_utena;

#[derive(Debug, Parser)]
struct Args {
    #[clap(short, long)]
    /// magia baiser listen addr
    listen: SocketAddr,

    #[clap(long)]
    /// record db dir
    record_dir: PathBuf,

    #[clap(long, action)]
    /// more magia baiser words~
    debug: bool,
}

pub async fn run() -> anyhow::Result<()> {
    let args = Args::parse();
    init_log(args.debug);

    let key_pair = KeyPair::generate_ed25519().unwrap();
    let config = Arc::new(Config {
        auth_banner: Some(HiiragiUtena::BANNER),
        auth_rejection_time: Duration::from_millis(100),
        inactivity_timeout: Some(Duration::from_secs(3)),
        keys: vec![key_pair],
        ..Default::default()
    });

    let mahou_syouzyo_record = MahouSyouzyoRecord::new(&args.record_dir).await?;
    let hiiragi_utena = HiiragiUtena::new(args.debug, mahou_syouzyo_record);

    info!("hey hey hey~~~");

    server::run(config, args.listen, hiiragi_utena).await?;

    Err(anyhow::anyhow!("magia baiser is defeated (╥_╥)"))
}

pub fn init_log(debug: bool) {
    let layer = fmt::layer()
        .pretty()
        .with_target(true)
        .with_writer(io::stderr);

    let level = if debug {
        LevelFilter::DEBUG
    } else {
        LevelFilter::INFO
    };

    let layered = Registry::default().with(layer).with(level);

    subscriber::set_global_default(layered).unwrap();
}
