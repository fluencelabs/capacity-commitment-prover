use std::sync::Mutex;

use ccp_cu_rpc_client::ccp_collector_rpc::collector_server::CollectorServer;
use clap::Parser;
use eyre::Context;
use tokio::net::UnixListener;
use tokio_stream::wrappers::UnixListenerStream;
use tonic::{transport::Server, Request, Response, Status};
use tracing_subscriber::filter::Directive;
use tracing_subscriber::EnvFilter;

use ccp_cu_rpc_client::ccp_collector_rpc::ReportRequest;
use ccp_cu_rpc_client::ccp_collector_rpc::ReportResponse;

// same as ccp_main crate
const CCP_LOG_ENV_VAR: &str = "CCP_LOG";

#[derive(Parser)]
struct Args {
    socket_path: String,
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let args = Args::parse();

    let filter = EnvFilter::builder()
        .with_env_var(CCP_LOG_ENV_VAR)
        .with_default_directive(Directive::from(tracing::Level::INFO))
        .from_env_lossy();
    let subscriber = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_writer(std::io::stderr)
        .with_thread_ids(true)
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting global tracing subscriber");

    run_server(&args.socket_path).await.expect("valid server");
}

#[derive(Default)]
struct CuProofCollector {
    proofs: Mutex<Vec<ReportRequest>>,
}

#[tonic::async_trait]
impl ccp_cu_rpc_client::ccp_collector_rpc::collector_server::Collector for CuProofCollector {
    async fn report_proof(
        &self,
        request: Request<ReportRequest>,
    ) -> Result<Response<ReportResponse>, Status> {
        let proof = request.into_inner();
        tracing::info!("Got proof {proof:?}");
        self.proofs.lock().expect("invalid mutex").push(proof);
        let reply = ReportResponse {};
        Ok(Response::new(reply))
    }
}

async fn run_server(uds_path: &str) -> eyre::Result<()> {
    prepare_fs(uds_path).await?;

    let collector = CuProofCollector::default();

    let uds = UnixListener::bind(uds_path)?;
    let uds_stream = UnixListenerStream::new(uds);

    Server::builder()
        .add_service(CollectorServer::new(collector))
        .serve_with_incoming(uds_stream)
        .await?;

    Ok(())
}

async fn prepare_fs(path: &str) -> eyre::Result<()> {
    use std::path::Path;

    let parent_dir = Path::new(path).parent().expect("malformed path");
    tokio::fs::create_dir_all(&parent_dir)
        .await
        .with_context(|| format!("failed to create dir {parent_dir:?}"))?;

    // old socket has to be removed; one can connect only to fresh socket
    ensure_removed(path).await
}

async fn ensure_removed(path: &str) -> eyre::Result<()> {
    use std::io::ErrorKind;

    let remove_res = tokio::fs::remove_file(path).await;
    match remove_res {
        Err(err) if err.kind() != ErrorKind::NotFound => {
            Err(err).with_context(|| format!("failed to remove file {path:?}"))
        }
        _ => Ok(()),
    }
}
