use std::collections::HashMap;
use std::net::SocketAddr;

use async_trait::async_trait;
use derive_more::Debug;
use russh::server::{Auth, Handler, Msg, Server, Session};
use russh::{Channel, ChannelId};
use tokio::io::AsyncWriteExt;
use tracing::{debug, info, instrument};

const MAX_CHANNEL: u8 = 16;

#[derive(Debug, Default)]
pub struct HiiragiUtena {}

impl HiiragiUtena {
    pub const BANNER: &'static str = "oh~~~? mahou syouzyo? let me torture you~~\n";

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
    mahou_syouzyo_auth: Option<(String, String)>,
    #[allow(dead_code)]
    peer: Option<SocketAddr>,
    #[debug(skip)]
    mahou_syouzyo_list: HashMap<ChannelId, Channel<Msg>>,
    channel_count: u8,
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

        Ok((self, Auth::Accept))
    }

    async fn channel_open_session(
        mut self,
        channel: Channel<Msg>,
        session: Session,
    ) -> Result<(Self, bool, Session), Self::Error> {
        self.channel_count += 1;
        if self.channel_count > MAX_CHANNEL {
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
                // avoid heap alloc, it equals format!("baka mahou syouzyo `{user}` ~ I got your secret~ `{password}`"
                writer.write_all(b"baka mahou syouzyo `").await?;
                writer.write_all(user.as_bytes()).await?;
                writer.write_all(b"` ~ I got your secret~ `").await?;
                writer.write_all(password.as_bytes()).await?;
                writer.write_all(b"`").await?;
            }

            Some(data) => {
                // avoid heap alloc, it equals
                // format!("baka mahou syouzyo `{user}` ~ I got your secret~ `{password}`, want to do this `{data}`~?"
                writer.write_all(b"baka mahou syouzyo `").await?;
                writer.write_all(user.as_bytes()).await?;
                writer.write_all(b"` ~ I got your secret~ `").await?;
                writer.write_all(password.as_bytes()).await?;

                writer.write_all(b"`, want to do this `").await?;
                writer.write_all(data).await?;
                writer.write_all(b"`~?").await?;
            }
        }

        writer.flush().await?;
        channel.eof().await?;
        channel.close().await?;

        Ok(())
    }
}
