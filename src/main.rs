use clap::Parser;
use fetcher::{FetchReceiverState, FetchResult, FetchResultReceiver};
use futures::lock::{Mutex, MutexGuard};
use serde_json::json;
use std::{
    io::{self, BufRead},
    sync::Arc,
};
use url_receiver::UrlReceiver;

mod fetcher;
mod url_receiver;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// paralellism
    #[arg(short, long, default_value_t = 1)]
    parallel: usize,
}

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

#[tokio::main]
async fn main() {
    env_logger::init();
    let args = Args::parse();

    let stdin = io::stdin();

    let (result_sender, result_receiver) = async_channel::unbounded::<FetchResult>();

    let mut fetch_result_receiver = FetchResultReceiver {
        receiver: result_receiver,
        state: Arc::new(Mutex::new(FetchReceiverState {
            results: vec![],
            start: None,
            end: None,
        })),
    };

    let (url_sender, url_receiver) = async_channel::unbounded::<String>();

    let mut fetch_result_receiver2 = fetch_result_receiver.clone();
    let fetch_results_handle = tokio::spawn(async move { fetch_result_receiver2.rcv().await });

    let url_receiver = UrlReceiver {
        receiver: url_receiver,
        result_sender,
    };

    let mut url_receivers = tokio::task::JoinSet::new();

    for _ in 0..args.parallel {
        let url_receiver = url_receiver.clone();
        url_receivers.spawn(async move { url_receiver.rcv().await });
    }

    fetch_result_receiver.start().await;

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

    fetch_result_receiver.end().await;

    log::debug!("Awaiting fetch results handle {fetch_results_handle:?}");
    drop(url_receiver);
    let fetch_result = fetch_results_handle.await.unwrap().unwrap();
    let state: MutexGuard<FetchReceiverState> = fetch_result.state.lock().await;
    let summary = crate::fetcher::summary(&state).unwrap();
    log::debug!("Summary: {summary:?}");
    if summary.valid_requests > 0 {
        println!("{}", json!(summary));
    } else {
        // json! macro panics on 0 valid requests. Who knows why?
        println!(
            "{}",
            json!({ "invalid_requests": summary.invalid_requests, "valid_requests": 0})
        );
    }
}
