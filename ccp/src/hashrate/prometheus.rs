/*
 * Copyright 2024 Fluence DAO
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

use std::sync::Arc;
use std::sync::Mutex;

use axum::body;
use axum::extract::State;
use axum::http;
use axum::response;
use axum::response::ErrorResponse;
use axum::routing::get;
use prometheus_client::registry::Registry;
use tokio::net::ToSocketAddrs;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

use super::HashrateCollector;
use crate::StorageMetrics;

#[derive(Clone, Default, Debug)]
pub(crate) struct PrometheusMetrics {
    pub(crate) hashrate_collector: Arc<Mutex<HashrateCollector>>,
    pub(crate) state_storage_metrics: Arc<StorageMetrics>,
    pub(crate) proof_storage_metrics: Arc<StorageMetrics>,
}

async fn handler_404() -> impl response::IntoResponse {
    (http::StatusCode::NOT_FOUND, "No such endpoint")
}

async fn handle_metrics(
    State(state): State<PrometheusMetrics>,
) -> response::Result<http::Response<body::Body>> {
    use prometheus_client::registry::Unit;

    let mut buf = String::new();

    {
        let mut registry = Registry::with_prefix("ccp");

        {
            let guard = state.hashrate_collector.lock().map_err(|_| {
                // TODO use parking_lot mutex
                log::error!("Prometehus metrics lock is poisoned");
                ErrorResponse::from(http::StatusCode::INTERNAL_SERVER_ERROR)
            })?;

            guard.apply_to_registry(&mut registry);
        }

        {
            let state_sub_registry =
                registry.sub_registry_with_label(("storage".into(), "state".into()));
            state_sub_registry.register(
                "io_error",
                "I/O errors for saving data",
                state.state_storage_metrics.io_error.clone(),
            );
            state_sub_registry.register_with_unit(
                "writing_time",
                "I/O errors for saving data",
                Unit::Seconds,
                state.state_storage_metrics.writing_time.clone(),
            );
        }

        {
            let proof_sub_registry =
                registry.sub_registry_with_label(("storage".into(), "proof".into()));
            proof_sub_registry.register(
                "io_error",
                "I/O errors for saving data",
                state.proof_storage_metrics.io_error.clone(),
            );
            proof_sub_registry.register_with_unit(
                "writing_time",
                "I/O errors for saving data",
                Unit::Seconds,
                state.proof_storage_metrics.writing_time.clone(),
            );
        }

        prometheus_client::encoding::text::encode(&mut buf, &registry).map_err(|e| {
            log::warn!("Metrics encode error: {}", e);
            ErrorResponse::from(http::StatusCode::INTERNAL_SERVER_ERROR)
        })?;
    }

    let body = body::Body::from(buf);
    http::Response::builder()
        .header(
            http::header::CONTENT_TYPE,
            "application/openmetrics-text; version=1.0.0; charset=utf-8",
        )
        .body(body)
        .map_err(|e| {
            log::warn!("Could not create prometheus response: {}", e);
            ErrorResponse::from(http::StatusCode::INTERNAL_SERVER_ERROR)
        })
}

async fn run_prometheus_endpoint(
    prometheus_listen_address: impl ToSocketAddrs + std::fmt::Debug,
    hashrate_collector: Arc<Mutex<HashrateCollector>>,
    cancellation: CancellationToken,
    state_storage_metrics: Arc<StorageMetrics>,
    proof_storage_metrics: Arc<StorageMetrics>,
) -> tokio::io::Result<()> {
    let state = PrometheusMetrics {
        hashrate_collector,
        state_storage_metrics,
        proof_storage_metrics,
    };
    let app = axum::Router::new()
        .route("/metrics", get(handle_metrics))
        .fallback(handler_404)
        .with_state(state);
    log::info!("Starting a prometheus endpoint at {prometheus_listen_address:?}");
    let listener = tokio::net::TcpListener::bind(&prometheus_listen_address)
        .await
        .inspect_err(|e| {
            log::error!(
                "Failed to start a prometheus endpoint at {prometheus_listen_address:?}: {e}"
            );
        })?;
    let server = axum::serve(listener, app.into_make_service());
    server
        .with_graceful_shutdown(cancellation.cancelled_owned())
        .await?;

    Ok(())
}

pub(crate) struct PrometheusEndpoint {
    cancellation: CancellationToken,
    handle: JoinHandle<tokio::io::Result<()>>,
}

impl PrometheusEndpoint {
    pub(crate) fn new(
        prometheus_listen_address: impl ToSocketAddrs + std::fmt::Debug + Send + Sync + 'static,
        hashrate_collector: Arc<Mutex<HashrateCollector>>,
        state_storage_metrics: Arc<StorageMetrics>,
        proof_storage_metrics: Arc<StorageMetrics>,
    ) -> Self {
        let cancellation = CancellationToken::new();

        let handle = tokio::task::spawn(run_prometheus_endpoint(
            prometheus_listen_address,
            hashrate_collector,
            cancellation.clone(),
            state_storage_metrics,
            proof_storage_metrics,
        ));

        Self {
            cancellation,
            handle,
        }
    }

    pub(crate) async fn shutdown(&mut self) -> tokio::io::Result<()> {
        log::info!("Shutting down Prometheus endpoint");
        self.cancellation.cancel();
        (&mut self.handle).await?
    }
}
