use anyhow::Result;
use futures::lock::Mutex;
use serde::Serialize;
use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};

#[derive(Clone, Debug)]
pub struct FetchResult {
    pub valid: bool,
    pub duration: Duration,
    pub status_code: u16,
    pub size: usize,
}

#[derive(Clone, Debug, Serialize)]
pub struct RunSummary {
    pub test_duration: u128,
    pub valid_requests: u128,
    pub invalid_requests: u128,
    pub average_request_duration: u128,
    pub shortest_request_duration: u128,
    pub longest_request_duration: u128,
    pub total_downloaded: usize,
    pub status_codes: HashMap<u16, u128>,
}

pub fn summary(state: &FetchReceiverState) -> Result<RunSummary> {
    let mut total_durations = 0u128;
    let mut status_codes: HashMap<u16, u128> = HashMap::new();
    let mut shortest_request_duration = u128::MAX;
    let mut longest_request_duration = 0u128;
    let mut total_downloaded = 0usize;
    let mut valid_requests = 0;
    let mut invalid_requests = 0;

    for result in &state.results {
        if !result.valid {
            invalid_requests = invalid_requests + 1;
            continue;
        } else {
            valid_requests = valid_requests + 1;
        }
        let duration = result.duration.as_millis();
        total_durations += duration;
        total_downloaded += result.size;
        shortest_request_duration = std::cmp::min(shortest_request_duration, duration);
        longest_request_duration = std::cmp::max(longest_request_duration, duration);
        status_codes.insert(
            result.status_code,
            match status_codes.get(&result.status_code) {
                Some(count) => count + 1,
                None => 1u128,
            },
        );
    }
    log::debug!("Valid requests: {valid_requests}");
    let summary = RunSummary {
        test_duration: state
            .end
            .unwrap()
            .duration_since(state.start.unwrap())
            .as_millis(),
        valid_requests,
        invalid_requests,
        average_request_duration: if valid_requests == 0 {
            0
        } else {
            (total_durations as f64 / valid_requests as f64) as u128
        },
        shortest_request_duration,
        longest_request_duration,
        total_downloaded,
        status_codes,
    };

    log::debug!("Got here. Sumamry is {:?}", summary);

    Ok(summary)
}

#[derive(Clone, Debug)]
pub struct FetchReceiverState {
    pub results: Vec<FetchResult>,
    pub start: Option<Instant>,
    pub end: Option<Instant>,
}

#[derive(Clone, Debug)]
pub struct FetchResultReceiver {
    pub receiver: async_channel::Receiver<FetchResult>,
    pub state: Arc<Mutex<FetchReceiverState>>,
}

impl FetchResultReceiver {
    pub async fn start(&mut self) {
        self.state.lock().await.start = Some(Instant::now());
    }

    pub async fn end(&mut self) {
        self.state.lock().await.end = Some(Instant::now());
    }

    pub async fn rcv(&mut self) -> Result<Self> {
        loop {
            match self.receiver.recv().await {
                Ok(result) => {
                    log::debug!("Got state {:?}", result);
                    self.state.lock().await.results.push(result);
                }
                Err(e) => {
                    log::info!("{e:?}");
                    break;
                }
            }
        }
        Ok(self.clone())
    }
}
