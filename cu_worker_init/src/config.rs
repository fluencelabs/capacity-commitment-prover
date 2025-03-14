use std::path::PathBuf;

use envconfig::Envconfig;

use ccp_shared::types::Difficulty;
use ccp_shared::types::GlobalNonce;
use ccp_shared::types::CUID;

/// Subset of main config for ccp-cu-main worker.

#[derive(Envconfig)]
pub struct TaskParameters {
    pub global_nonce: GlobalNonce,
    pub difficulty: Difficulty,
    pub cu_id: CUID,
    pub temp_dir: PathBuf,
    pub override_logical_cpu_list: Option<String>,
}
