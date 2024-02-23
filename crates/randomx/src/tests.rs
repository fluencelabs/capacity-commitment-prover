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

use crate::Cache;
use crate::Dataset;
use crate::RandomXFlags;
use crate::RandomXVM;
use crate::ResultHash;

fn run_light_randomx(global_nonce: &[u8], local_nonce: &[u8], flags: RandomXFlags) -> ResultHash {
    let cache = Cache::new(&global_nonce, flags).unwrap();
    let vm = RandomXVM::light(cache, flags).unwrap();
    vm.hash(&local_nonce)
}

fn run_fast_randomx(global_nonce: &[u8], local_nonce: &[u8], flags: RandomXFlags) -> ResultHash {
    let dataset = Dataset::new(&global_nonce, flags).unwrap();
    let vm = RandomXVM::fast(dataset, flags).unwrap();
    vm.hash(&local_nonce)
}

#[test]
fn light_mode_works() {
    let global_nonce = vec![1, 2, 3, 4, 5, 6, 7];
    let local_nonce = vec![2, 3, 4, 5, 6, 7];
    let flags = RandomXFlags::recommended();

    let actual_result = run_light_randomx(&global_nonce, &local_nonce, flags);
    let expected_result = ResultHash::from_slice([
        67, 239, 84, 18, 247, 8, 93, 182, 61, 251, 183, 153, 67, 84, 87, 218, 135, 14, 249, 163,
        31, 190, 15, 90, 57, 60, 80, 138, 37, 182, 122, 35,
    ]);

    assert_eq!(actual_result, expected_result);
}

#[test]
fn fast_mode_works() {
    let global_nonce = vec![1, 2, 3, 4, 5, 6, 7];
    let local_nonce = vec![2, 3, 4, 5, 6, 7];
    let flags = RandomXFlags::recommended_full_mem();

    let actual_result = run_fast_randomx(&global_nonce, &local_nonce, flags);
    let expected_result = ResultHash::from_slice([
        67, 239, 84, 18, 247, 8, 93, 182, 61, 251, 183, 153, 67, 84, 87, 218, 135, 14, 249, 163,
        31, 190, 15, 90, 57, 60, 80, 138, 37, 182, 122, 35,
    ]);

    assert_eq!(actual_result, expected_result);
}

#[test]
fn fast_equals_to_light() {
    use rand::RngCore;

    let mut global_nonce = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut global_nonce);

    let mut local_nonce = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut local_nonce);

    let light_flags = RandomXFlags::recommended();
    let light_result = run_light_randomx(&global_nonce, &local_nonce, light_flags);

    let fast_flags = RandomXFlags::recommended_full_mem();
    let fast_result = run_fast_randomx(&global_nonce, &local_nonce, fast_flags);

    assert_eq!(fast_result, light_result);
}
