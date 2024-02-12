use sha3::digest::core_api::CoreWrapper;
use sha3::digest::CtOutput;
use sha3::Keccak256Core;

use ccp_shared::types::GlobalNonce;
use ccp_shared::types::CUID;

/// Computes global nonce specific for a particular CU by
/// keccak(global_nonce + cu_id)
pub fn compute_global_nonce_cu(global_nonce: &GlobalNonce, cu_id: &CUID) -> CtOutput<CoreWrapper<Keccak256Core>> {
    use sha3::Digest;

    let mut hasher = sha3::Keccak256::new();
    hasher.update(global_nonce);
    hasher.update(cu_id);

    hasher.finalize().into()
}
