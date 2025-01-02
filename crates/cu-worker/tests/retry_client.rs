use std::sync::Arc;
use std::time::Duration;

use ccp_cu::EpochParameters;
use ccp_cu::RawProof;
use ccp_cu_rpc_client::ccp_collector_rpc::collector_server::Collector;
use ccp_cu_rpc_client::ccp_collector_rpc::collector_server::CollectorServer;
use ccp_cu_rpc_client::ccp_collector_rpc::ReportResponse;
use ccp_cu_rpc_client::ReportRequest;
use ccp_cu_worker::retry_client::ProofRetryClient;
use ccp_shared::types::ResultHash;
use tokio::select;
use tokio::sync::mpsc::Sender;
use tokio::sync::Notify;
use tokio_util::sync::CancellationToken;

#[tokio::test]
async fn test_normal() {
    let listen_addr = gen_endpoint_address();
    let endpoint_url = format!("grpc://{listen_addr}");
    let mut proofs = gen_test_proofs();

    let (proof_sender, mut proof_receiver) = tokio::sync::mpsc::channel(100);
    let (_stats_sender, stats_receiver) = tokio::sync::mpsc::channel(100);

    let (server_cancel, server_stopped_notify) = start_server(proof_sender, listen_addr.clone());
    let (client, proof_sender) = ProofRetryClient::new(endpoint_url, stats_receiver);

    let next_proof = proofs.next().unwrap();
    proof_sender.send(next_proof.clone()).await.unwrap();

    let client_cancel = CancellationToken::new();
    let client_cancel_cloned = client_cancel.clone();

    tokio::spawn(async move {
        select! {
            _ = client_cancel_cloned.cancelled() => {},
            _ = client.run() => {},
        }
    });

    let received_proof = proof_receiver.recv().await.unwrap();

    assert_eq!(received_proof, next_proof);

    client_cancel.cancel();
    server_cancel.cancel();
    server_stopped_notify.notified().await;
}

#[tokio::test]
async fn test_failing_server() {
    let listen_addr = gen_endpoint_address();
    let endpoint_url = format!("grpc://{listen_addr}");
    let mut proofs = gen_test_proofs();

    let (sender, mut proof_receiver) = tokio::sync::mpsc::channel(100);
    let (_stats_sender, stats_receiver) = tokio::sync::mpsc::channel(100);

    let (client, proof_sender) = ProofRetryClient::new(endpoint_url, stats_receiver);

    let next_proof_1 = proofs.next().unwrap();
    proof_sender.send(next_proof_1.clone()).await.unwrap();

    let client_cancel = CancellationToken::new();
    let client_cancel_cloned = client_cancel.clone();

    tokio::spawn(async move {
        select! {
            _ = client_cancel_cloned.cancelled() => {},
            _ = client.run() => {},
        }
    });

    tokio::time::sleep(Duration::from_secs(20)).await;

    let (server_cancel, server_stopped_notify) = start_server(sender.clone(), listen_addr.clone());
    let received_proof = proof_receiver.recv().await.unwrap();
    assert_eq!(received_proof, next_proof_1);

    server_cancel.cancel();
    server_stopped_notify.notified().await;

    let next_proof_2 = proofs.next().unwrap();
    proof_sender.send(next_proof_2.clone()).await.unwrap();

    tokio::time::sleep(Duration::from_secs(20)).await;

    assert!(proof_receiver.try_recv().is_err());

    let (server_cancel, server_stopped_notify) = start_server(sender, listen_addr.clone());
    let received_proof = proof_receiver.recv().await.unwrap();
    assert_eq!(received_proof, next_proof_2);
    assert!(proof_receiver.try_recv().is_err());

    server_cancel.cancel();
    server_stopped_notify.notified().await;

    client_cancel.cancel();
}

#[derive(Debug, Clone)]
struct TestServer {
    sender: Sender<RawProof>,
}

impl TestServer {
    fn new(sender: Sender<RawProof>) -> Self {
        Self { sender }
    }
}

#[tonic::async_trait]
impl Collector for TestServer {
    async fn report(
        &self,
        request: tonic::Request<ReportRequest>,
    ) -> std::result::Result<tonic::Response<ReportResponse>, tonic::Status> {
        let req = request.into_inner();
        for rpc_proof in &req.proofs {
            let raw_proof = rpc_proof.try_into().unwrap();
            self.sender.try_send(raw_proof).unwrap();
        }

        let reply = ReportResponse {};
        Ok(tonic::Response::new(reply))
    }
}

fn gen_endpoint_address() -> String {
    use rand::prelude::*;

    let mut rng = rand::thread_rng();
    let port = rng.gen_range(40000..60000);
    format!("127.0.0.1:{}", port)
}

fn start_server(sender: Sender<RawProof>, listen_addr: String) -> (CancellationToken, Arc<Notify>) {
    let server = TestServer::new(sender.clone());
    let cancel = CancellationToken::new();
    let shutdown_notify = Arc::new(Notify::new());

    let server_cancel = cancel.clone();
    let server_shutdown_notify = shutdown_notify.clone();

    let _handle = tokio::spawn(async move {
        let _ = tonic::transport::Server::builder()
            .add_service(CollectorServer::new(server))
            .serve_with_shutdown(listen_addr.parse().unwrap(), server_cancel.cancelled())
            .await
            .inspect_err(|e| {
                tracing::error!("CCP endpoint error: {e}");
            });
        server_shutdown_notify.notify_waiters();
    });

    (cancel, shutdown_notify)
}

fn gen_test_proofs() -> impl Iterator<Item = RawProof> {
    let epoch = EpochParameters {
        global_nonce: "13dcfdbd51f89c893a726149c63ea2320741c3db9204410d9ea568033d40ebe6"
            .parse()
            .unwrap(),
        difficulty: "0001142ed3232df44227771034bbb1877ec02be3cef9fe40dc4c3b67e2fd8bbb"
            .parse()
            .unwrap(),
    };
    let local_nonce = "535bd78966878d2ea041da3c576ba2f5fa1703701c32dcf5eab5467afec35ec8"
        .parse()
        .unwrap();
    let cu_id = "4782b0b6ecce93e2ea76761d6d22113a86cba81f93d36dbc078d893572558205"
        .parse()
        .unwrap();
    let result_hash = "000000000000000005285335df677fe3da96a911e23271c02e12e958dab4623d"
        .parse::<ResultHash>()
        .unwrap()
        .into_slice()
        .to_owned();

    (0u64..).map(move |n| {
        let mut new_result_hash = result_hash.clone();
        (&mut new_result_hash)[0..8].copy_from_slice(&n.to_be_bytes()[..]);

        RawProof {
            epoch: epoch.clone(),
            local_nonce: Clone::clone(&local_nonce),
            cu_id: Clone::clone(&cu_id),
            result_hash: ResultHash::from_slice(new_result_hash),
        }
    })
}
