use anyhow::Result;
use std::time::{Duration, Instant};

use crate::fetcher::FetchResult;

#[derive(Clone)]
pub struct UrlReceiver {
    pub receiver: async_channel::Receiver<String>,
    pub result_sender: async_channel::Sender<FetchResult>,
}

impl UrlReceiver {
    pub async fn rcv(self) -> Result<()> {
        while let Ok(url) = self.receiver.recv().await {
            let before = Instant::now();
            let result = match reqwest::Client::new().get(url).send().await {
                Ok(client) => {
                    let after = Instant::now();
                    let status_code = client.status().as_u16();
                    let size = client.bytes().await?.len();
                    FetchResult {
                        valid: true,
                        duration: after.duration_since(before),
                        status_code,
                        size,
                    }
                }
                Err(e) => {
                    log::warn!("Err: {e:?}");
                    FetchResult {
                        valid: false,
                        duration: Duration::from_millis(0),
                        status_code: 0,
                        size: 0,
                    }
                }
            };
            self.result_sender.send(result).await?;
        }
        log::debug!("Channel closed, returning");
        Ok(())
    }
}
