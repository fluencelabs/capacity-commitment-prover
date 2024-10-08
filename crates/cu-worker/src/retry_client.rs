use std::pin::pin;
use std::time::Duration;

use backoff::backoff::Backoff;
use backoff::ExponentialBackoffBuilder;
use ccp_cu::RawProof;
use ccp_cu_rpc_client::CcpCuRpcHttpClient;
use eyre::Context;
use eyre::OptionExt;
use tokio::select;
use tokio::sync::mpsc::Receiver;
use tokio::sync::mpsc::Sender;

use crate::transport::UpstreamTransport;

pub struct RetryClient {
    rpc_endpoint: String,
    receiver: Receiver<RawProof>,
}

const BUFFER_SIZE: usize = 1000;

impl RetryClient {
    pub fn new(rpc_endpoint: String) -> (Self, Sender<RawProof>) {
        let (sender, receiver) = tokio::sync::mpsc::channel(BUFFER_SIZE);
        let self_ = RetryClient {
            rpc_endpoint,
            receiver,
        };
        (self_, sender)
    }

    pub async fn run(mut self) -> eyre::Result<()> {
        let mut collected_proofs = vec![];

        loop {
            assert!(collected_proofs.is_empty());
            let raw_proof = self
                .receiver
                .recv()
                .await
                .ok_or_eyre("No more input data")?;
            collected_proofs.push(raw_proof);

            // Retry loop: untill the request succeeds.
            let mut backoff_gen = ExponentialBackoffBuilder::new()
                .with_initial_interval(Duration::from_millis(100))
                .with_max_interval(Duration::from_secs(5))
                .build();
            loop {
                // We create a new client instance because if the endpoint restarts and Unix domain socket is used,
                // the file is recreated.
                let result = self.try_sending_report(&collected_proofs).await;
                let error = match result {
                    Ok(_) => {
                        // We have successfuly sent data upstream, clean the buffer and break the retry loop.
                        collected_proofs.clear();
                        break;
                    }
                    Err(e) => e,
                };

                let next_backoff = backoff_gen.next_backoff().unwrap();
                tracing::error!(
                    "failed to send {count} proofs upstream: {error}, retrying in {next_backoff:?}",
                    count = collected_proofs.len(),
                );

                self.wait_for_backoff_collecting(next_backoff, &mut collected_proofs)
                    .await;
            }
        }
    }

    async fn try_sending_report(&self, collected_proofs: &[RawProof]) -> eyre::Result<()> {
        let mut rpc_client = CcpCuRpcHttpClient::auto(self.rpc_endpoint.clone())
            .await
            .context("invalid RPC endpoint")?;
        UpstreamTransport::report_proof(&mut rpc_client, collected_proofs).await?;
        Ok(())
    }

    async fn wait_for_backoff_collecting(
        &mut self,
        next_backoff: Duration,
        collected_proofs: &mut Vec<RawProof>,
    ) {
        let deadline = tokio::time::sleep(next_backoff);
        let mut deadline = pin!(deadline);
        loop {
            select! {
                raw_proof = self.receiver.recv() => {
                    if let Some(raw_proof) = raw_proof {
                        collected_proofs.push(raw_proof);
                    } else {
                        // even if the channel is closed, we still have to send data after the backoff
                        deadline.await;
                        break;
                    }
                }
                _ = &mut deadline => {
                    break;
                }
            }
        }
    }
}
