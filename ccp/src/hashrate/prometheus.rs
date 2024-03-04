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

#[derive(Clone, Default, Debug)]
pub(crate) struct PrometheusMetrics {
    pub(crate) hashrate_collector: Arc<Mutex<HashrateCollector>>,
}

async fn handler_404() -> impl response::IntoResponse {
    (http::StatusCode::NOT_FOUND, "No such endpoint")
}

async fn handle_metrics(
    State(state): State<PrometheusMetrics>,
) -> response::Result<http::Response<body::Body>> {
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
) -> tokio::io::Result<()> {
    let state = PrometheusMetrics { hashrate_collector };
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
    ) -> Self {
        let cancellation = CancellationToken::new();

        let handle = tokio::task::spawn(run_prometheus_endpoint(
            prometheus_listen_address,
            hashrate_collector,
            cancellation.clone(),
        ));

        Self {
            cancellation,
            handle,
        }
    }

    pub(crate) async fn stop(self) -> tokio::io::Result<()> {
        self.cancellation.cancel();
        self.handle.await?
    }
}
