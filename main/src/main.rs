/*
 * Copyright 2024 Fluence Labs Limited
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

#![warn(rust_2018_idioms)]
#![warn(rust_2021_compatibility)]
#![deny(
    dead_code,
    nonstandard_style,
    unused_imports,
    unused_mut,
    unused_variables,
    unused_unsafe,
    unreachable_patterns
)]

use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;

use clap::ArgAction;
use clap::Parser;
use eyre::WrapErr as _;
use tokio::sync::Mutex;

use capacity_commitment_prover::CCProver;
use ccp_config::CCPConfig;
use ccp_config::ThreadsPerCoreAllocationPolicy;
use ccp_randomx::RandomXFlags;
use ccp_rpc_server::CCPRcpHttpServer;
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
    utility_core_id: u32,

    #[arg(long)]
    threads_per_physical_core: std::num::NonZeroUsize,

    #[arg(long)]
    proof_dir: PathBuf,

    #[arg(long)]
    state_dir: PathBuf,

    #[arg(long, action = ArgAction::SetTrue)]
    enable_msr: bool,
}

fn main() -> eyre::Result<()> {
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

    check_writable_dir(&args.prover_args.proof_dir)
        .wrap_err("The --proof-dir value should be a writeable directory path")?;
    check_writable_dir(&args.prover_args.state_dir).wrap_err(
        "The --state-dir value should be a writeable directory path",
    )?;

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

async fn async_main(bind_address: String, prover_args: ProverArgs) -> eyre::Result<()> {
    // Build a prover
    let prover = build_prover(prover_args).await?;
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

async fn build_prover(prover_args: ProverArgs) -> eyre::Result<CCProver> {
    // TODO an option?
    let randomx_flags = RandomXFlags::recommended_full_mem();

    let config = CCPConfig {
        thread_allocation_policy: ThreadsPerCoreAllocationPolicy::Exact {
            threads_per_physical_core: prover_args.threads_per_physical_core,
        },
        randomx_flags,
        proof_dir: prover_args.proof_dir,
        state_dir: prover_args.state_dir,
        enable_msr: prover_args.enable_msr,
    };

    CCProver::from_saved_state(prover_args.utility_core_id.into(), config)
        .await
        // e doesn't implement Sync, and cannot be converted to anyhow::Error or eyre::Error.
        // as it will be reported to a user immediately, convert the error to string
        .map_err(|e| eyre::eyre!(e.to_string()))
}

// Preliminary check that is useful on early diagnostics.
fn check_writable_dir(path: &Path) -> eyre::Result<()> {
    if !path.is_dir() {
        eyre::bail!("{path:?} is not a directory");
    }
    let meta = std::fs::metadata(path)?;
    let perm = meta.permissions();
    if perm.readonly() {
        eyre::bail!("{path:?} is not writable");
    }
    Ok(())
}
