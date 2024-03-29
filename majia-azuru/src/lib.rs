use std::error::Error;
use std::future::ready;
use std::io::{stdout, Write};
use std::net::{SocketAddr, ToSocketAddrs};
use std::num::NonZeroUsize;
use std::time::Instant;

use async_ssh2_tokio::{AuthMethod, Client, ServerCheckMethod};
use clap::Parser;
use futures_util::{stream, StreamExt, TryStreamExt};
use tabwriter::TabWriter;

#[derive(Debug, Parser)]
pub struct Args {
    #[clap(short, long)]
    /// concurrent fall to evil
    concurrent: NonZeroUsize,

    #[clap(short, long)]
    /// max fall to evil count
    max: usize,

    #[clap(short, long)]
    /// ssh username
    user: String,

    #[clap(short, long)]
    /// ssh password
    password: String,

    #[clap(short = '4', long, action)]
    /// only use ipv4
    ipv4: bool,

    #[clap(short = '6', long, action)]
    /// only use ipv6
    ipv6: bool,

    /// ssh server addr
    addr: String,

    /// ssh server port
    #[clap(short = 'P', long, default_value_t = 22)]
    port: u16,

    /// command sent to ssh server
    command: Vec<String>,
}

pub async fn run() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    if args.ipv4 && args.ipv6 {
        return Err("ipv4 and ipv6 flag only can set one".into());
    }

    let command = args.command.join(" ");

    let addr = (args.addr, args.port).to_socket_addrs()?;
    let addr: Vec<_> = match (args.ipv4, args.ipv6) {
        (false, false) => addr.collect(),
        (true, false) => addr.filter(|addr| addr.is_ipv4()).collect(),
        (false, true) => addr.filter(|addr| addr.is_ipv6()).collect(),
        _ => unreachable!(),
    };

    let mut tasks = Vec::with_capacity(args.max);
    for _ in 0..args.max {
        tasks.push(async {
            let client = connect(&args.user, &args.password, &addr).await?;
            fall_to_evil(client, &command).await
        });
    }

    let start = Instant::now();
    let tasks = stream::iter(tasks).buffer_unordered(args.concurrent.get());
    tasks.try_for_each(|_| ready(Ok(()))).await?;

    let use_time = start.elapsed();
    let feps = args.max as f64 / use_time.as_secs_f64();

    let mut writer = TabWriter::new(stdout());
    writeln!(writer, "max fall to evil:\t{}", args.max)?;
    writeln!(writer, "concurrent fall to evil:\t{}", args.concurrent)?;
    writeln!(writer, "feps(fall to evil per second):\t{feps}")?;
    writeln!(writer, "use time:\t{use_time:?}")?;

    writer.flush()?;

    Ok(())
}

async fn connect(
    user: &str,
    password: &str,
    addr: &[SocketAddr],
) -> Result<Client, Box<dyn Error>> {
    let auth_method = AuthMethod::with_password(password);

    let client = Client::connect(addr, user, auth_method, ServerCheckMethod::NoCheck).await?;

    Ok(client)
}

async fn fall_to_evil(client: Client, command: &str) -> Result<(), Box<dyn Error>> {
    match client.execute(command).await {
        Err(
            err @ async_ssh2_tokio::Error::KeyAuthFailed
            | err @ async_ssh2_tokio::Error::PasswordWrong,
        ) => Err(err.into()),
        Err(async_ssh2_tokio::Error::KeyInvalid(err)) => Err(err.into()),
        Err(async_ssh2_tokio::Error::AddressInvalid(err)) => Err(err.into()),

        _ => Ok(()),
    }
}
