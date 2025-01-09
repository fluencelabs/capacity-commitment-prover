/*
 * Copyright 2024 Fluence DAO
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

use ccp::stats_collecting_thread::StatsSink;
use ccp_cu::proving_thread::sync::to_stats_collecting_message::ToStatsCollectingInlet;
use ccp_cu::proving_thread::sync::to_stats_collecting_message::ToStatsCollectingMessage;
use ccp_cu::proving_thread::sync::to_stats_collecting_message::ToStatsCollectingOutlet;

const RPC_STATS_QUEUE_SIZE: usize = 1024;

pub struct RpcStatsCollector {
    sender: ToStatsCollectingInlet,
}

impl RpcStatsCollector {
    pub fn new() -> (Self, ToStatsCollectingOutlet) {
        // TODO queue size
        let (sender, receiver) = tokio::sync::mpsc::channel(RPC_STATS_QUEUE_SIZE);
        (Self { sender }, receiver)
    }
}

impl StatsSink for RpcStatsCollector {
    fn handle_stats(&self, record: &ToStatsCollectingMessage) {
        tracing::trace!("stats message: {record:?}");
        let r = self.sender.try_send(record.clone());
        if let Err(err) = r {
            tracing::error!("failed to send stats record to a worker: {err}; ignoring");
        }
    }
}
