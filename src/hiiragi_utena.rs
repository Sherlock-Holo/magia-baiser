use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use bytes::Buf;
use derive_more::Debug;
use futures_util::future::try_join_all;
use russh::server::{Auth, Config, Handler, Msg, Server, Session};
use russh::{server, Channel, ChannelId};
use russh_keys::key::KeyPair;
use tokio::io::AsyncWriteExt;
use tracing::{debug, info, instrument};

use crate::defeat_record::MahouSyouzyoRecord;

const MAX_CHANNEL: usize = 16;

#[derive(Debug, Clone)]
pub struct HiiragiUtena {
    mahou_syouzyo_record: MahouSyouzyoRecord,
}

impl HiiragiUtena {
    pub const BANNER: &'static str = "oh~~~? mahou syouzyo? let me torture you~~~\n";

    pub fn new(mahou_syouzyo_record: MahouSyouzyoRecord) -> Self {
        Self {
            mahou_syouzyo_record,
        }
    }

    pub async fn run(&mut self, addr: &[SocketAddr]) -> anyhow::Result<()> {
        let key_pair = KeyPair::generate_ed25519().unwrap();
        let config = Arc::new(Config {
            auth_banner: Some(HiiragiUtena::BANNER),
            auth_rejection_time: Duration::from_millis(100),
            inactivity_timeout: Some(Duration::from_secs(3)),
            keys: vec![key_pair],
            ..Default::default()
        });

        let tasks = addr
            .iter()
            .map(|&addr| {
                let hiiragi_utena = self.clone();
                let config = config.clone();

                tokio::spawn(async move { server::run(config, addr, hiiragi_utena).await })
            })
            .map(|task| async { task.await.unwrap() });
        try_join_all(tasks).await?;

        Ok(())
    }

    fn hensin(&self) -> MagiaBaiser {
        MagiaBaiser {
            mahou_syouzyo_auth: None,
            peer: None,
            mahou_syouzyo_list: Default::default(),
            mahou_syouzyo_record: self.mahou_syouzyo_record.clone(),
        }
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

#[derive(Debug)]
pub struct MagiaBaiser {
    mahou_syouzyo_auth: Option<(String, String)>,
    #[allow(dead_code)]
    peer: Option<SocketAddr>,
    #[debug(skip)]
    mahou_syouzyo_list: HashMap<ChannelId, Channel<Msg>>,
    #[debug(skip)]
    mahou_syouzyo_record: MahouSyouzyoRecord,
}

#[async_trait]
impl Handler for MagiaBaiser {
    type Error = russh::Error;

    #[instrument(level = "debug", skip(user, password))]
    async fn auth_password(
        mut self,
        user: &str,
        password: &str,
    ) -> Result<(Self, Auth), Self::Error> {
        info!("baka mahou syouzyo `{user}` ~ I got your secret~ `{password}`");

        self.mahou_syouzyo_auth = Some((user.to_string(), password.to_string()));

        // ignore error
        let _ = self
            .mahou_syouzyo_record
            .add_mahou_syouzyo(user, password)
            .await;

        Ok((self, Auth::Accept))
    }

    async fn channel_open_session(
        mut self,
        channel: Channel<Msg>,
        session: Session,
    ) -> Result<(Self, bool, Session), Self::Error> {
        if self.mahou_syouzyo_list.len() > MAX_CHANNEL {
            return Err(russh::Error::Disconnect);
        }

        self.mahou_syouzyo_list.insert(channel.id(), channel);

        Ok((self, true, session))
    }

    async fn shell_request(
        mut self,
        channel_id: ChannelId,
        mut session: Session,
    ) -> Result<(Self, Session), Self::Error> {
        if let Some(channel) = self.mahou_syouzyo_list.remove(&channel_id) {
            self.laugh(channel_id, &mut session, channel, None).await?;
        }

        Ok((self, session))
    }

    async fn exec_request(
        mut self,
        channel_id: ChannelId,
        data: &[u8],
        mut session: Session,
    ) -> Result<(Self, Session), Self::Error> {
        if let Some(channel) = self.mahou_syouzyo_list.remove(&channel_id) {
            self.laugh(channel_id, &mut session, channel, Some(data))
                .await?;
        }

        Ok((self, session))
    }
}

impl MagiaBaiser {
    async fn laugh(
        &mut self,
        channel_id: ChannelId,
        session: &mut Session,
        channel: Channel<Msg>,
        data: Option<&[u8]>,
    ) -> Result<(), russh::Error> {
        session.channel_success(channel_id);
        let mut writer = channel.make_writer();

        let (user, password) = match &self.mahou_syouzyo_auth {
            None => {
                return Err(russh::Error::WrongChannel);
            }

            Some(auth) => (&auth.0, &auth.1),
        };

        match data {
            None => {
                info!(user, password, "mahou syouzyo want shell~");

                // avoid heap alloc, it equals format!("baka mahou syouzyo `{user}` ~ I got your secret~ `{password}`"
                let mut data = b"baka mahou syouzyo `"
                    .chain(user.as_bytes())
                    .chain(&b"` ~ I got your secret~ `"[..])
                    .chain(password.as_bytes())
                    .chain(&b"`"[..]);
                writer.write_all_buf(&mut data).await?;
            }

            Some(data) => {
                let exec_cmd = String::from_utf8_lossy(data);

                info!(user, password, %exec_cmd, "mahou syouzyo want do something~");

                // avoid heap alloc, it equals
                // format!("baka mahou syouzyo `{user}` ~ I got your secret~ `{password}`, want to do this `{data}`~?"
                let mut data = b"baka mahou syouzyo `"
                    .chain(user.as_bytes())
                    .chain(&b"` ~ I got your secret~ `"[..])
                    .chain(password.as_bytes())
                    .chain(&b"`, want to do this `"[..])
                    .chain(data)
                    .chain(&b"`~?"[..]);
                writer.write_all_buf(&mut data).await?;
            }
        }

        writer.flush().await?;
        channel.eof().await?;
        channel.close().await?;

        Ok(())
    }
}
