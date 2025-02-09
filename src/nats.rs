use async_nats::{Connection, Subscription};
use log::{error, warn};
use thiserror::Error;

type Result<T> = std::result::Result<T, self::Error>;

#[derive(Error, Debug)]
pub(crate) enum Error {
    #[error("Unable to connect to NATS: {0}")]
    UnableToConnect(std::io::Error),
    #[error("Unable to subscribe to channel ({0}): {1}")]
    UnableToSubscribe(String, std::io::Error),
}

#[derive(Clone)]
pub(crate) struct Conn {
    pub(crate) conn: Connection,
}

impl Conn {
    pub(crate) async fn new(nats_address: &str) -> Result<Conn> {
        let conn = async_nats::connect(nats_address)
            .await
            .map_err(Error::UnableToConnect)?;

        Ok(Conn { conn })
    }
    pub(crate) async fn new_with_user_pass(nats_address: &str, user: &str, pass: &str) -> Result<Conn> {
        let conn = async_nats::Options::with_user_pass(user, pass)
            .connect(nats_address)
            .await
            .map_err(Error::UnableToConnect)?;

        Ok(Conn { conn })
    }

    pub(crate) async fn send(&self, channel: &str, message: Vec<u8>) -> Result<()> {
        let mut retries: i8 = 0;
        let mut result = self.conn.publish(channel, &message).await;

        // keep retrying if publish fails
        while result.is_err() {
            retries += 1;
            let error_message = format!("Failed to send message, failed {} times", retries);

            // before 5 retries log as warning, as error after
            if retries < 5 {
                warn!("{}", &error_message)
            } else {
                error!("{}", &error_message)
            }

            result = self.conn.publish(channel, &message).await;
        }

        Ok(())
    }

    pub(crate) async fn subscribe(&self, channel: &str) -> Result<Subscription> {
        Ok(self
            .conn
            .subscribe(channel)
            .await
            .map_err(|e| Error::UnableToSubscribe(channel.to_string(), e))?)
    }
}
