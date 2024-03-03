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

use clap::Parser;
use eyre::WrapErr as _;
use tracing_subscriber::filter::Directive;
use tracing_subscriber::EnvFilter;

use capacity_commitment_prover::CCProver;
use ccp_config::load_config;
use ccp_config::CCPConfig;
use ccp_rpc_server::BackgroundFacade;
use ccp_rpc_server::CCPRcpHttpServer;

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long)]
    config_path: String,
}

fn main() -> eyre::Result<()> {
    let args = Args::parse();
    let config = load_config(args.config_path.as_str())?;

    let filter = EnvFilter::builder()
        .with_env_var("RUST_LOG")
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

    check_writable_dir(&config.state_dir)
        .wrap_err("state-dir value in a config should be a writeable directory path")?;

    let tokio_cores = config
        .http_server
        .utility_cores_ids
        .iter()
        .cloned()
        .map(Into::into)
        .collect::<Vec<_>>();

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .worker_threads(2)
        .on_thread_start(move || {
            let pid = std::thread::current().id();
            tracing::info!("Pinning tokio thread {pid:?} to cores {tokio_cores:?}");
            if !cpu_utils::pinning::pin_current_thread_to_cpuset(tokio_cores.iter().copied()) {
                tracing::error!("Tokio thread pinning failed");
            }
        })
        .build()
        .wrap_err("failed to build tokio runtime")?;

    runtime.block_on(async_main(config))
}

async fn async_main(config: CCPConfig) -> eyre::Result<()> {
    let bind_address = (config.http_server.host.clone(), config.http_server.port);

    tracing::info!("creating prover with config {config:?}");
    let prover = CCProver::from_saved_state(config)
        .await
        .map_err(|e| eyre::eyre!(e.to_string()))?;

    tracing::info!(
        "starting RPC server on {}:{}",
        bind_address.0,
        bind_address.1
    );
    let rpc_server = CCPRcpHttpServer::new(BackgroundFacade::new(prover));
    let server_handle = rpc_server
        .run_server(bind_address)
        .await
        .wrap_err("starting an RPC server failed")?;

    server_handle.stopped().await; // wait indefinitely

    Ok(())
}

// Preliminary check that is useful on early diagnostics.
fn check_writable_dir(path: &Path) -> eyre::Result<()> {
    if !path.is_dir() {
        eyre::bail!("{path:?} is not a directory");
    }

    let meta = std::fs::metadata(path)?;
    let permissions = meta.permissions();
    if permissions.readonly() {
        eyre::bail!("{path:?} is not writable");
    }

    Ok(())
}
