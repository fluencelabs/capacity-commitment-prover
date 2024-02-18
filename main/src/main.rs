use std::{path::PathBuf, sync::Arc};

use clap::Parser;
use eyre::WrapErr as _;
use tokio::sync::Mutex;

use capacity_commitment_prover::CCProver;
use ccp_config::{CCPConfig, ThreadsPerCoreAllocationPolicy};
use ccp_rpc_server::CCPRcpHttpServer;
use randomx_rust_wrapper::RandomXFlags;
use tracing_subscriber::EnvFilter;

#[derive(Parser, Debug)]
struct Args {
    #[arg(long)]
    bind_address: String,
    #[arg(long = "tokio-core-id")]
    tokio_core_ids: Vec<usize>,

    #[command(flatten)]
    prover_args: ProverArgs,
}

#[derive(Parser, Debug)]
struct ProverArgs {
    #[arg(long)]
    utility_core_id: usize,
    #[arg(long)]
    threads_per_physical_core: std::num::NonZeroUsize,

    #[arg(long)]
    dir_to_store_proofs: PathBuf,
    #[arg(long)]
    dir_to_store_persistent_state: PathBuf,
}

fn main() -> Result<(), eyre::Error> {
    let subscriber = tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_writer(std::io::stderr)
        .with_thread_ids(true)
        .finish();
    tracing::subscriber::set_global_default(subscriber)
        .wrap_err("setting global tracing subscriber failed")?;
    tracing_log::LogTracer::init()?;

    let args = Args::parse();
    tracing::info!("{args:?}");

    if args.tokio_core_ids.is_empty() {
        eyre::bail!("please, define at least one --tokio-core-id");
    }

    #[cfg(target_os = "linux")]
    let tokio_cores = args.tokio_core_ids;

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .worker_threads(2)
        .on_thread_start(move || {
            #[cfg(target_os = "linux")]
            {
                let pid = std::thread::current().id();
                tracing::info!("Pinning tokio thread {pid:?} to cores {tokio_cores:?}");
                affinity::set_thread_affinity(&tokio_cores)
                    .expect("failed to set tokio thread affinity");
            }
        })
        .build()
        .wrap_err("failed to build tokio runtime")?;

    runtime.block_on(async_main(args.bind_address, args.prover_args))
}

async fn async_main(bind_address: String, prover_args: ProverArgs) -> Result<(), eyre::Error> {
    // Build a prover
    let prover = build_prover(prover_args);
    tracing::info!("created prover");

    // Launch RPC API
    let rpc_server = CCPRcpHttpServer::new(Arc::new(Mutex::new(prover)));
    tracing::info!("starting an RPC server");
    let server_handle = rpc_server
        .run_server(bind_address)
        .await
        .wrap_err("starting an RPC server failed")?;
    tracing::info!("the RPC server started");

    server_handle.stopped().await; // wait indefinitely

    Ok(())
}

fn build_prover(prover_args: ProverArgs) -> CCProver {
    // TODO an option?
    let randomx_flags = RandomXFlags::recommended_full_mem();

    let config = CCPConfig {
        thread_allocation_policy: ThreadsPerCoreAllocationPolicy::Exact {},
        randomx_flags,
        dir_to_store_proofs: prover_args.dir_to_store_proofs,
        dir_to_store_persistent_state: prover_args.dir_to_store_persistent_state,
    };

    CCProver::new(prover_args.utility_core_id, config)
}
