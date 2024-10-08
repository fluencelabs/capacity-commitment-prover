use std::path::PathBuf;

use clap::Parser;
use envconfig::Envconfig;
use eyre::WrapErr as _;
use tokio::{select, sync::mpsc};
use tokio_util::sync::CancellationToken;
use tracing_subscriber::filter::Directive;
use tracing_subscriber::EnvFilter;

use ccp_cu::proving_thread::sync::to_state_operating_message::ToStateOperatingOutlet;
use ccp_cu::proving_thread::sync::to_stats_collecting_message::ToStatsCollectingOutlet;
use ccp_cu::{CUProver, CUProverConfig};
use ccp_cu_worker::retry_client::RetryClient;
use ccp_msr::MSRModeEnforcer;
use ccp_shared::types::{Difficulty, EpochParameters, GlobalNonce, CUID};

// same as ccp_main crate
const CCP_LOG_ENV_VAR: &str = "CCP_LOG";

#[derive(Envconfig)]
struct TaskParameters {
    global_nonce: GlobalNonce,
    difficulty: Difficulty,
    cu_id: CUID,
    upstream: String,
    temp_dir: PathBuf,
}

#[derive(Parser)]
struct Args {
    #[arg(help = "YAML config file")]
    config: Option<String>,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> eyre::Result<()> {
    use ccp_config::cu_config::load_cu_config;

    let args = Args::parse();
    let config = load_cu_config(args.config.as_deref())?;

    let filter = EnvFilter::builder()
        .with_env_var(CCP_LOG_ENV_VAR)
        .with_default_directive(Directive::from(config.logs.log_level))
        .from_env_lossy();
    let subscriber = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_writer(std::io::stderr)
        .with_thread_ids(true)
        .finish();

    tracing::subscriber::set_global_default(subscriber)
        .wrap_err("setting global tracing subscriber failed")?;
    tracing_log::LogTracer::init()?;

    let prover_config = CUProverConfig::new(config.optimizations, config.workers);
    let parameters: TaskParameters = Envconfig::init_from_env()?;

    let (to_stats_collecting, from_stats_collecting) = mpsc::channel(config.utility_queue_size);
    let (to_state_operating, from_state_operating) = mpsc::channel(config.utility_queue_size);

    // dummy disabled instance
    let msr_enforcer = MSRModeEnforcer::from_os(false);
    let core_id = None;

    let mut cu_prover = CUProver::create(
        prover_config,
        to_state_operating,
        to_stats_collecting,
        msr_enforcer,
        core_id,
        parameters.temp_dir,
    )
    .await
    // error is not Send, convert it to a string
    .map_err(|e| eyre::eyre!(format!("{e:?}")))?;

    let dummy_cancellation_token = CancellationToken::new();
    let epoch = EpochParameters::new(parameters.global_nonce, parameters.difficulty);
    cu_prover
        .new_epoch(epoch, parameters.cu_id, dummy_cancellation_token)
        .await
        // error is not Send, convert it to a string
        .map_err(|e| eyre::eyre!(format!("{e:?}")))?;

    message_dispatching_loop(
        from_stats_collecting,
        from_state_operating,
        parameters.upstream,
    )
    .await
}

async fn message_dispatching_loop(
    mut from_stats_collecting: ToStatsCollectingOutlet,
    mut from_state_operating: ToStateOperatingOutlet,
    rpc_endpoint: String,
) -> eyre::Result<()> {
    let (client, sender) = RetryClient::new(rpc_endpoint);
    let _client_handle = tokio::task::spawn(client.run());
    loop {
        select! {
                _ = from_stats_collecting.recv() => {}
                msg = from_state_operating.recv() => {
                    use ccp_cu::proving_thread::sync::to_state_operating_message::ToStateOperatingMessage::*;
                    match msg.expect("worker thread failed") {
                        ProofFound { proof } => {
                            tracing::info!("sending received proof upstream: {proof:?}");
                            if let Err(e) = sender.send(proof).await {
                                tracing::error!("failed to send proof: {e}")
                            }
                        },
                        ErrorHappened { core_id: _core_id, error } => {
                            unreachable!("no error reported because of no valid core_id: {error:?}");
                        },
                    }
                }
        }
    }
}
