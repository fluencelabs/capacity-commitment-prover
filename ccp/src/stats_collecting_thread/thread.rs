/*
 * Copyright 2024 Fluence Labs Limited
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time;

use tokio_util::sync::CancellationToken;

use super::UTResult;
use crate::cu::proving_thread::sync::to_stats_collecting_message::{
    ToStatsCollectingInlet, ToStatsCollectingMessage, ToStatsCollectingOutlet,
};
use crate::hashrate::HashrateHandler;

const CUMULATIVE_HASHRATE_UPDATE_INTERVAL: u64 = 60;

pub(crate) struct StatsCollecting {
    to_stats_collecting: ToStatsCollectingInlet,
    cancellation: CancellationToken,
    handle: tokio::task::JoinHandle<()>,
}

impl StatsCollecting {
    pub(crate) fn spawn(
        hashrate_handler: HashrateHandler,
        stats_collecting_queue_size: usize,
    ) -> Self {
        let (to_stats_collecting, from_stats_collecting) =
            mpsc::channel(stats_collecting_queue_size);

        let cancellation = CancellationToken::new();

        let ut_impl = StatsCollectingImpl::new(
            from_stats_collecting,
            cancellation.clone(),
            hashrate_handler,
        );

        let handle = tokio::spawn(ut_impl.utility_closure());
        Self {
            to_stats_collecting,
            cancellation,
            handle,
        }
    }

    pub(crate) async fn shutdown(&mut self) -> UTResult<()> {
        log::info!("Shutting down utility thread");
        self.cancellation.cancel();
        Ok((&mut self.handle).await?)
    }

    pub(crate) fn get_to_stats_collecting_channel(&self) -> ToStatsCollectingInlet {
        self.to_stats_collecting.clone()
    }
}

struct StatsCollectingImpl {
    to_stats_collecting: ToStatsCollectingOutlet,
    cancellation: CancellationToken,
    hashrate_handler: HashrateHandler,
}

impl StatsCollectingImpl {
    pub(crate) fn new(
        to_stats_collecting: ToStatsCollectingOutlet,
        cancellation: CancellationToken,
        hashrate_handler: HashrateHandler,
    ) -> Self {
        Self {
            to_stats_collecting,
            cancellation,
            hashrate_handler,
        }
    }

    pub(crate) async fn utility_closure(mut self) {
        use futures::FutureExt;
        use futures::StreamExt;

        let mut cum_hashrate_ticker =
            time::interval(Duration::from_secs(CUMULATIVE_HASHRATE_UPDATE_INTERVAL));

        #[cfg(feature = "crossterm")]
        let mut terminal_event_reader = crossterm::event::EventStream::new();

        #[cfg(not(feature = "crossterm"))]
        let mut terminal_event_reader = futures::stream::pending::<()>();

        loop {
            tokio::select! {
                Some(utility_message) = self.to_stats_collecting.recv() => self.handle_to_stats_collecting_message(utility_message).await,
                _ = cum_hashrate_ticker.tick() => self.handle_cum_hashrate_tick().await,
                maybe_event = terminal_event_reader.next().fuse() => self.handle_terminal_event(maybe_event).await,
                _ = self.cancellation.cancelled() => {
                    log::info!("The utility thread was shutdown");
                    return;
                }
            }
        }
    }

    async fn handle_to_stats_collecting_message(&mut self, message: ToStatsCollectingMessage) {
        use ToStatsCollectingMessage::*;

        match message {
            ProofFound { core_id } => {
                self.hashrate_handler.proof_found(core_id);
            }
            Hashrate(record) => {
                log::info!("{record}");

                if let Err(error) = self.hashrate_handler.account_record(record) {
                    log::error!("hashrate accounting failed: {error}");
                }
            }
            Epoch(epoch) => {
                if let Err(error) = self.hashrate_handler.handle_epoch(epoch) {
                    log::error!("epoch processing failed: {error}");
                }
            }
        }
    }

    async fn handle_cum_hashrate_tick(&mut self) {
        if let Err(error) = self.hashrate_handler.handle_cum_tick() {
            log::error!("cumulative hashrate tick failed: {error}");
        }
    }

    #[cfg(feature = "crossterm")]
    async fn handle_terminal_event(
        &mut self,
        maybe_event: Option<Result<crossterm::event::Event, std::io::Error>>,
    ) {
        use crossterm::event::Event;
        use crossterm::event::KeyCode;
        use itertools::Itertools;

        if let Some(Ok(event)) = maybe_event {
            if event == Event::Key(KeyCode::Enter.into()) {
                let sliding_hashrate = self.hashrate_handler.sliding_hashrate();
                if sliding_hashrate.is_empty() {
                    println!("no hashrate for the last 900 secs,\nCCP is either busy with initialization or idle");
                    return;
                }

                println!(
                    "{0: <10} | {1: <10} | {2: <10} | {3: <10}",
                    "core id", "10 secs", "60 secs", "900 secs"
                );

                for (core_id, thread_hashrate) in sliding_hashrate
                    .iter()
                    .sorted_by_key(|(&core_id, _)| core_id)
                {
                    println!(
                        "{0: <10} | {1: <10.2} | {2: <10.2} | {3: <10.2}",
                        core_id,
                        thread_hashrate.window_10.compute_hashrate(),
                        thread_hashrate.window_60.compute_hashrate(),
                        thread_hashrate.window_900.compute_hashrate()
                    );
                }
            }
        }
    }

    #[cfg(not(feature = "crossterm"))]
    async fn handle_terminal_event(&mut self, _maybe_event: Option<()>) {}
}
