use std::net::SocketAddr;
use std::sync::Arc;

use async_trait::async_trait;
use dashmap::DashMap;
use derive_more::Debug;
use russh::server::{Auth, Handler, Msg, Server};
use russh::{Channel, ChannelId};
use russh_keys::key::PublicKey;
use tracing::{debug, info, instrument};

#[derive(Debug, Default)]
pub struct HiiragiUtena {}

impl HiiragiUtena {
    fn hensin(&self) -> MagiaBaiser {
        MagiaBaiser::default()
    }
}

#[async_trait]
impl Server for HiiragiUtena {
    type Handler = MagiaBaiser;

    fn new_client(&mut self, peer_addr: Option<SocketAddr>) -> Self::Handler {
        debug!(mahou_syouzyo=?peer_addr, "wuhoo~ catch mahou syouzyo");

        self.hensin()
    }
}

#[derive(Debug, Default)]
pub struct MagiaBaiser {
    #[allow(dead_code)]
    peer: Option<SocketAddr>,
    #[debug(skip)]
    mahou_syouzyo_list: Arc<DashMap<ChannelId, Channel<Msg>>>,
}

#[async_trait]
impl Handler for MagiaBaiser {
    type Error = russh::Error;

    #[instrument(level = "debug", skip(user, password))]
    async fn auth_password(self, user: &str, password: &str) -> Result<(Self, Auth), Self::Error> {
        info!("baka mahou syouzyo `{user}` ~ I got your secret~ `{password}`");

        Err(russh::Error::NotAuthenticated)
    }

    #[instrument(level = "debug", skip(user, public_key))]
    async fn auth_publickey(
        self,
        user: &str,
        public_key: &PublicKey,
    ) -> Result<(Self, Auth), Self::Error> {
        info!("baka mahou syouzyo `{user}` ~, so excited~, got it~ `{public_key:?}`");

        Err(russh::Error::NotAuthenticated)
    }
}
