use std::io;
use std::net::SocketAddr;
use std::path::PathBuf;

use clap::Parser;
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
    listen: Vec<SocketAddr>,

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

    let mahou_syouzyo_record = MahouSyouzyoRecord::new(&args.record_dir).await?;

    info!("hey hey hey~~~");

    let mut hiiragi_utena = HiiragiUtena::new(mahou_syouzyo_record);

    hiiragi_utena.run(&args.listen).await?;

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
