use anyhow::Result;
use clap::Parser;
use serde::Serialize;
use serde_json::json;
use std::{
    io::{self, BufRead},
    //sync::{Arc, Mutex},
    time::{Duration, Instant},
};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// paralellism
    #[arg(short, long, default_value_t = 1)]
    parallel: usize,
}

#[derive(Clone)]
struct UrlReceiver {
    pub receiver: async_channel::Receiver<String>,
    pub result_sender: async_channel::Sender<FetchResult>,
}

impl UrlReceiver {
    pub async fn rcv(self) -> Result<()> {
        while let Ok(url) = self.receiver.recv().await {
            let before = Instant::now();
            let client = match reqwest::Client::new().get(url).send().await {
                Ok(result) => result,
                Err(e) => {
                    log::error!("Err: {e:?}");
                    break;
                }
            };
            let after = Instant::now();
            let status_code = client.status().as_u16();
            let size = client.bytes().await?.len();
            self.result_sender
                .send(FetchResult {
                    duration: after.duration_since(before),
                    status_code,
                    size,
                })
                .await?;
        }
        log::debug!("Channel closed, returning");
        Ok(())
    }
}

#[derive(Clone, Debug, Serialize)]
struct FetchResult {
    pub duration: Duration,
    pub status_code: u16,
    pub size: usize,
}

struct FetchResultReceiver {
    receiver: async_channel::Receiver<FetchResult>,
    results: Vec<FetchResult>,
    start: Option<Instant>,
    end: Option<Instant>,
}

impl FetchResultReceiver {
    pub fn start(&mut self) -> () {
        self.start = Some(Instant::now());
    }

    pub fn end(&mut self) -> () {
        self.end = Some(Instant::now());
    }

    pub async fn rcv(&mut self) -> Result<Vec<FetchResult>> {
        loop {
            match self.receiver.recv().await {
                Ok(result) => {
                    log::info!("{}", json!(result));
                    self.results.push(result);
                }
                Err(e) => {
                    log::info!("{e:?}");
                    break;
                }
            }
        }
        Ok(self.results.clone()) // get rid of this clone.
    }
}

#[tokio::main]
async fn main() {
    env_logger::init();
    let args = Args::parse();

    let stdin = io::stdin();

    let (result_sender, result_receiver) = async_channel::unbounded::<FetchResult>();

    let mut fetch_result_receiver = FetchResultReceiver {
        receiver: result_receiver,
        results: vec![],
        start: None,
        end: None,
    };

    let (url_sender, url_receiver) = async_channel::unbounded::<String>();

    let fetch_results_handle = tokio::spawn(async move { fetch_result_receiver.rcv().await });

    let url_receiver = UrlReceiver {
        receiver: url_receiver,
        result_sender,
    };

    let mut url_receivers = tokio::task::JoinSet::new();

    for _ in 0..args.parallel {
        let url_receiver = url_receiver.clone();
        url_receivers.spawn(async move { url_receiver.rcv().await });
    }

    for line in stdin.lock().lines() {
        let line = line.unwrap();
        log::debug!("Sending URL {line}");
        url_sender.send(line).await.unwrap();
    }

    log::debug!("{} send channel(s) open", url_sender.sender_count());
    url_sender.close();
    log::debug!("All lines sent, url_sender closed");

    let mut url_results = vec![];
    while let Some(url_receiver) = url_receivers.join_next().await {
        log::debug!("{url_receiver:?} became ready");
        url_results.push(url_receiver.unwrap());
    }

    let after = Instant::now();

    log::debug!("Awaiting fetch results handle {fetch_results_handle:?}");
    drop(url_receiver);
    let fetch_result = fetch_results_handle.await.unwrap().unwrap();
    log::info!("Rsult: {:?}", fetch_result);
}
