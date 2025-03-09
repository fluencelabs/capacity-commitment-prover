use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex as StdMutex;

use clap::Parser;
use cpu_utils::cpusets::parse_cpu_list;
use cpu_utils::cpusets::read_cpu_list_string;
use cpu_utils::NonEmpty;
use envconfig::Envconfig;
use eyre::OptionExt;
use eyre::WrapErr as _;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;
use tracing_subscriber::filter::Directive;
use tracing_subscriber::EnvFilter;

use ccp::hashrate::HashrateCollector;
use ccp::hashrate::HashrateHandler;
use ccp::stats_collecting_thread::StatsCollectingThread;
use ccp_cu::proving_thread::sync::to_state_operating_message::ToStateOperatingOutlet;
use ccp_cu::proving_thread::sync::to_stats_collecting_message::ToStatsCollectingOutlet;
use ccp_cu::CUProver;
use ccp_cu::CUProverConfig;
use ccp_cu::CoreIdAllocation;
use ccp_cu_worker::metrics::RpcStatsCollector;
use ccp_cu_worker::retry_client::ProofRetryClient;
use ccp_msr::MSRModeEnforcer;
use ccp_shared::types::Difficulty;
use ccp_shared::types::EpochParameters;
use ccp_shared::types::GlobalNonce;
use ccp_shared::types::CUID;

// same as ccp_main crate
const CCP_LOG_ENV_VAR: &str = "CCP_LOG";

const STATS_QUEUE_SIZE: usize = 1024;

#[derive(Envconfig)]
struct TaskParameters {
    global_nonce: GlobalNonce,
    difficulty: Difficulty,
    cu_id: CUID,
    upstream: String,
    temp_dir: PathBuf,
    override_logical_cpu_list: Option<String>,
    // 0-1000
    oom_score_adj: Option<u16>,
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

    if let Some(oom_score_adj) = parameters.oom_score_adj {
        let res = update_oom_score_adj(oom_score_adj);
        if let Err(err) = res {
            tracing::warn!("Failed to update oom_score_adj: {err:?}");
        }
    }

    let (to_state_operating, from_state_operating) = mpsc::channel(config.utility_queue_size);

    let core_allocation = get_core_allocation_from_sys(parameters.override_logical_cpu_list);
    tracing::info!("Core allocation: {core_allocation}");

    let msr_enforcer = MSRModeEnforcer::from_os(false);
    let dummy_cancellation_token = CancellationToken::new();
    let epoch = EpochParameters::new(parameters.global_nonce, parameters.difficulty);

    let hashrate_collector = Arc::new(StdMutex::new(HashrateCollector::new()));
    let hashrate_handler = HashrateHandler::new(
        hashrate_collector.clone(),
        None,
        config.logs.report_hashrate,
    )?;

    let (collector, collector_inlet) = RpcStatsCollector::new();

    let stats_collecting_thread = StatsCollectingThread::spawn(
        hashrate_handler,
        STATS_QUEUE_SIZE,
        config.logs.cumulative_hashrate_update_interval_secs,
        vec![Box::new(collector)],
    );

    let mut cu_prover = CUProver::create(
        prover_config,
        to_state_operating,
        stats_collecting_thread.get_to_stats_collecting_channel(),
        msr_enforcer,
        core_allocation,
        parameters.temp_dir,
    )
    .await
    // the error is not Send, convert it to a string
    .map_err(|e| eyre::eyre!(format!("{e:?}")))?;

    cu_prover
        .new_epoch(epoch, parameters.cu_id, dummy_cancellation_token)
        .await
        // the error is not Send, convert it to a string
        .map_err(|e| eyre::eyre!(format!("{e:?}")))?;

    message_dispatching_loop(from_state_operating, collector_inlet, parameters.upstream).await
}

async fn message_dispatching_loop(
    mut from_state_operating: ToStateOperatingOutlet,
    to_stats_rpc_client: ToStatsCollectingOutlet,
    rpc_endpoint: String,
) -> eyre::Result<()> {
    let (client, proof_sender) = ProofRetryClient::new(rpc_endpoint, to_stats_rpc_client);
    let _client_handle = tokio::task::spawn(client.run());

    loop {
        use ccp_cu::proving_thread::sync::to_state_operating_message::ToStateOperatingMessage::*;

        let msg = from_state_operating
            .recv()
            .await
            .ok_or_eyre("worker thread failed, terminating")?;

        match msg {
            ProofFound { proof } => {
                tracing::info!("sending received proof upstream: {proof:?}");
                if let Err(e) = proof_sender.send(proof).await {
                    tracing::error!("failed to send proof: {e}")
                }
            }
            ErrorHappened { error, .. } => {
                unreachable!("no error reported because of no valid core_id: {error:?}");
            }
        }
    }
}

fn get_core_allocation_from_sys(overriding_core_allocation: Option<String>) -> CoreIdAllocation {
    let cpu_ids_string = overriding_core_allocation
        .map(Ok)
        .unwrap_or_else(read_cpu_list_string);

    let core_ids = match cpu_ids_string {
        Ok(cpu_list_str) => parse_cpu_list(&cpu_list_str),
        Err(e) => {
            tracing::error!("failed to load a cgroup cpuset CPU list, using single thread: {e}");
            return CoreIdAllocation::SingleUnrestricted;
        }
    };
    let core_ids = match core_ids {
        Ok(cpu_list) => cpu_list,
        Err(e) => {
            tracing::error!("failed to parse a cgroup cpuset CPU list, using single thread: {e}");
            return CoreIdAllocation::SingleUnrestricted;
        }
    };

    match NonEmpty::from_vec(core_ids) {
        Some(logical_core_ids) => CoreIdAllocation::LogicalCoreIds(logical_core_ids),
        None => CoreIdAllocation::SingleUnrestricted,
    }
}

fn update_oom_score_adj(oom_score_adj: u16) -> std::io::Result<()> {
    let pid = std::process::id();
    let path = format!("/proc/{pid}/oom_score_adj");

    std::fs::write(path, &oom_score_adj.to_string())
}
