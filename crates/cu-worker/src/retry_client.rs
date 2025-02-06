use std::pin::pin;
use std::time::Duration;

use backoff::backoff::Backoff;
use backoff::ExponentialBackoffBuilder;
use ccp_cu::proving_thread::sync::to_stats_collecting_message::ToStatsCollectingMessage;
use ccp_cu::proving_thread::sync::to_stats_collecting_message::ToStatsCollectingOutlet;
use ccp_cu::RawProof;
use ccp_cu_rpc_client::CcpCuRpcHttpClient;
use ccp_cu_rpc_client::HashRate;
use eyre::Context;
use tokio::select;
use tokio::sync::mpsc::Receiver;
use tokio::sync::mpsc::Sender;

use crate::transport::ProofUpstreamTransport;

pub struct ProofRetryClient {
    rpc_endpoint: String,
    proof_receiver: Receiver<RawProof>,
    stats_receiver: Receiver<ToStatsCollectingMessage>,
}

const BUFFER_SIZE: usize = 1000;
const INITIAL_BACKOFF_INTERVAL_SECS: u64 = 10;
const MAX_BACKOFF_INTERVAL_SECS: u64 = 600;

impl ProofRetryClient {
    pub fn new(
        rpc_endpoint: String,
        stats_receiver: ToStatsCollectingOutlet,
    ) -> (Self, Sender<RawProof>) {
        let (proof_sender, proof_receiver) = tokio::sync::mpsc::channel(BUFFER_SIZE);
        let self_ = ProofRetryClient {
            rpc_endpoint,
            proof_receiver,
            stats_receiver,
        };
        (self_, proof_sender)
    }

    pub async fn run(mut self) {
        let mut collected_proofs = vec![];
        let mut collected_stats: Vec<HashRate> = vec![];

        loop {
            if !collected_stats.is_empty() || !collected_proofs.is_empty() {
                tracing::error!("internal ERROR: either collected stats or collected proofs aren't empty after the cycle: {collected_stats:?}, {collected_proofs:?}");
                collected_stats.clear();
                collected_proofs.clear();
            }

            select! {
                raw_proof = self.proof_receiver.recv() => {
                    match raw_proof {
                        Some(raw_proof) => collected_proofs.push(raw_proof),
                        // nothing to read
                        None => {
                            tracing::warn!("RETRY CLIENT: the proof channel is closed by sender, exiting the retrying loop");
                            return
                        },
                    }
                }
                stats = self.stats_receiver.recv() => {
                    if let Some(stats) = stats {
                        // Error means here that stats record is not to be sent by gRPC
                        if let Ok(stats) = (&stats).try_into() {
                            collected_stats.push(stats);
                        }
                    }
                }
            }

            // Retry loop: until the request succeeds.
            let mut backoff_gen = ExponentialBackoffBuilder::new()
                .with_initial_interval(Duration::from_secs(INITIAL_BACKOFF_INTERVAL_SECS))
                .with_max_interval(Duration::from_secs(MAX_BACKOFF_INTERVAL_SECS))
                .with_max_elapsed_time(None)
                .build();
            loop {
                // We create a new client instance because if the endpoint restarts and Unix domain socket is used,
                // the file is recreated.
                let result = self
                    .try_sending_report(&collected_proofs, &collected_stats)
                    .await;
                let error = match result {
                    Ok(_) => {
                        // We have successfuly sent data upstream, clean the buffer and break the retry loop.
                        collected_proofs.clear();
                        collected_stats.clear();
                        break;
                    }
                    Err(e) => e,
                };
                tracing::error!("failed to send connect to lightmare gRPC: {error:?}, retrying...",);

                let next_backoff = backoff_gen.next_backoff().unwrap_or_else(|| Duration::from_secs(MAX_BACKOFF_INTERVAL_SECS));
                tracing::error!("retrying in {next_backoff:?}");

                self.wait_for_backoff_collecting(
                    next_backoff,
                    &mut collected_proofs,
                    &mut collected_stats,
                )
                .await;
            }
        }
    }

    async fn try_sending_report(
        &self,
        collected_proofs: &[RawProof],
        collected_stats: &[HashRate],
    ) -> eyre::Result<()> {
        let mut rpc_client = CcpCuRpcHttpClient::auto(self.rpc_endpoint.clone())
            .await
            .context("invalid RPC endpoint")?;
        ProofUpstreamTransport::report(&mut rpc_client, collected_proofs, collected_stats).await?;
        Ok(())
    }

    async fn wait_for_backoff_collecting(
        &mut self,
        next_backoff: Duration,
        collected_proofs: &mut Vec<RawProof>,
        collected_stats: &mut Vec<HashRate>,
    ) {
        let deadline = tokio::time::sleep(next_backoff);
        let mut deadline = pin!(deadline);
        loop {
            select! {
                raw_proof = self.proof_receiver.recv() => {
                    if let Some(raw_proof) = raw_proof {
                        collected_proofs.push(raw_proof);
                    } else {
                        // even if the channel is closed, we still have to send data after the backoff
                        deadline.await;
                        break;
                    }
                }
                stats = self.stats_receiver.recv() => {
                    if let Some(stats) = stats {
                        if let Ok(stats) = (&stats).try_into() {
                            collected_stats.push(stats);
                        }
                    }
                }
                _ = &mut deadline => {
                    break;
                }
            }
        }
    }
}
