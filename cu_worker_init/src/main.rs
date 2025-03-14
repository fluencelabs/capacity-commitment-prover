mod config;

use clap::Parser;
use cpu_utils::cpusets::parse_cpu_list;
use cpu_utils::cpusets::read_cpu_list_string;
use cpu_utils::NonEmpty;
use envconfig::Envconfig;
use eyre::WrapErr as _;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;
use tracing_subscriber::filter::Directive;
use tracing_subscriber::EnvFilter;

use crate::config::TaskParameters;
use ccp_cu::CUProver;
use ccp_cu::CUProverConfig;
use ccp_cu::CoreIdAllocation;
use ccp_msr::MSRModeEnforcer;
use ccp_shared::types::EpochParameters;

// same as ccp_main crate
const CCP_INIT_LOG_ENV_VAR: &str = "CCP_INIT_LOG";

#[derive(Parser)]
struct Args {
    #[arg(help = "YAML config file")]
    config: Option<String>,
}

fn main() -> eyre::Result<()> {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .max_blocking_threads(2)
        .thread_stack_size(512 * 1024)
        .build()
        .context("Internal error: failed to build tokio runtime")?;
    runtime.block_on(async_main())
}

async fn async_main() -> eyre::Result<()> {
    use ccp_config::cu_config::load_cu_config;

    let args = Args::parse();
    let config = load_cu_config(args.config.as_deref())?;

    let filter = EnvFilter::builder()
        .with_env_var(CCP_INIT_LOG_ENV_VAR)
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

    let core_allocation = get_core_allocation_from_sys(parameters.override_logical_cpu_list);
    tracing::info!("Core allocation: {core_allocation}");

    let msr_enforcer = MSRModeEnforcer::from_os(false);
    let dummy_cancellation_token = CancellationToken::new();
    let epoch = EpochParameters::new(parameters.global_nonce, parameters.difficulty);

    let (to_state_sender, _to_state_receiver) = mpsc::channel(128);
    let (stats_sender, _stats_receiver) = mpsc::channel(128);

    let mut cu_prover = CUProver::create(
        prover_config,
        to_state_sender,
        stats_sender,
        msr_enforcer,
        core_allocation,
        parameters.temp_dir,
    )
    .await
    // the error is not Send, convert it to a string
    .map_err(|e| eyre::eyre!(format!("{e:?}")))?;

    cu_prover
        .prepare_dataset(epoch, parameters.cu_id, dummy_cancellation_token)
        .await
        .map_err(|e| eyre::eyre!(format!("{e:?}")))?;

    Ok(())
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
