use ccp_cu::{CUProver, CUProverConfig};
use ccp_msr::MSRModeEnforcer;
use ccp_shared::types::{Difficulty, EpochParameters, GlobalNonce, CUID};

use clap::Parser;
use envconfig::Envconfig;
use tokio::{select, sync::mpsc};
use tokio_util::sync::CancellationToken;

#[derive(Envconfig)]
struct TaskParameters {
    global_nonce: GlobalNonce,
    difficulty: Difficulty,
    cu_id: CUID,
}

#[derive(Parser)]
struct Args {
    config: Option<String>,
}

// TODO configure tokio?
#[tokio::main]
async fn main() -> eyre::Result<()> {
    use ccp_config::cu_config::load_cu_config;

    let args = Args::parse();
    let config = load_cu_config(args.config.as_deref())?;

    let prover_config = CUProverConfig::new(config.optimizations, config.workers);
    let parameters: TaskParameters = Envconfig::init_from_env()?;

    let (to_stats_collecting, mut from_stats_collecting) = mpsc::channel(config.utility_queue_size);

    let (to_state_operating, mut from_state_operating) = mpsc::channel(config.utility_queue_size);

    // dummy disabled instance
    let msr_enforcer = MSRModeEnforcer::from_os(false);
    // TODO make it Option<T> in the cu prover
    let core_id = 0.into();

    let mut cu_prover = CUProver::create(
        prover_config,
        to_state_operating,
        to_stats_collecting,
        msr_enforcer,
        core_id,
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

    loop {
        select! {
                _ = from_stats_collecting.recv() => {}
                msg = from_state_operating.recv() => {
                    use ccp_cu::proving_thread::sync::to_state_operating_message::ToStateOperatingMessage::*;
                    match msg.expect("worker thread failed") {
                        ProofFound { proof } => {
                            todo!("send proof upstream")
                        },
                        ErrorHappened { core_id, error } => {
                            todo!("send error upstream: {error:?}")
                        },
                    }
                }
        }
    }
}
