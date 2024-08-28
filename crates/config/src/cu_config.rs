use serde::Deserialize;
use serde::Serialize;

use crate::defaults::default_utility_queue_size;
use crate::unresolved_config::*;
use crate::*;

/// Subset of main config for ccp-cu-main worker.

#[derive(Clone, Debug, PartialEq)]
pub struct CUWorkerConfig {
    pub optimizations: Optimizations,
    // TODO remove
    pub logs: Logs,
    pub workers: Workers,
    // TODO remove
    pub tokio: Tokio,
    pub utility_queue_size: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct UnresolvedCUWorkerConfig {
    #[serde(default)]
    pub optimizations: UnresolvedOptimizations,
    #[serde(default)]
    pub logs: UnresolvedLogs,
    #[serde(default)]
    pub workers: UnresolvedWorkers,
    #[serde(default)]
    pub tokio: UnresolvedTokio,
    #[serde(default = "default_utility_queue_size")]
    pub utility_queue_size: usize,
}

impl UnresolvedCUWorkerConfig {
    pub fn resolve(self) -> eyre::Result<CUWorkerConfig> {
        let optimizations = self.optimizations.resolve_for_cu_worker()?;
        let logs = self.logs.resolve();
        let workers = self.workers.resolve();
        let tokio = self.tokio.resolve();
        let utility_queue_size = self.utility_queue_size;

        let config = CUWorkerConfig {
            optimizations,
            logs,
            workers,
            tokio,
            utility_queue_size,
        };
        Ok(config)
    }
}

pub fn load_cu_config(path: Option<&str>) -> eyre::Result<CUWorkerConfig> {
    use ::config::Config;
    use ::config::File;
    use ::config::FileFormat;

    let mut config_builder = Config::builder();
    if let Some(path) = path {
        let config_source = File::with_name(path)
            .required(true)
            .format(FileFormat::Toml);
        config_builder = config_builder.add_source(config_source);
    }
    let config = config_builder.build()?;

    let config: UnresolvedCUWorkerConfig = config.try_deserialize()?;
    config.resolve()
}
