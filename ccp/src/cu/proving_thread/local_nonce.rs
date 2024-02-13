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

use rand::RngCore;

pub(crate) const LOCAL_NONCE_SIZE: usize = 32;

#[derive(Clone, Copy, Debug)]
pub(crate) struct LocalNonce {
    nonce: [u8; LOCAL_NONCE_SIZE],
}

impl LocalNonce {
    /// Creates a new random nonce.
    /// It uses random generator to be sure that in the next start with the same parameters,
    /// CCP won't do the same job twice.
    pub(crate) fn random() -> Self {
        let mut rng = rand::thread_rng();
        let mut nonce = [0u8; LOCAL_NONCE_SIZE];

        rng.fill_bytes(&mut nonce);

        Self { nonce }
    }

    /// Generates the next nonce.
    pub(crate) fn next(&mut self) {
        let mut nonce_as_u64: u64 = u64::from_le_bytes(
            self.nonce[0..std::mem::size_of::<u64>()]
                .try_into()
                .unwrap(),
        );
        nonce_as_u64 = nonce_as_u64.wrapping_add(1);
        self.nonce[0..std::mem::size_of::<u64>()].copy_from_slice(&u64::to_le_bytes(nonce_as_u64));
    }

    pub(crate) fn prev(&mut self) {
        let mut nonce_as_u64: u64 = u64::from_le_bytes(
            self.nonce[0..std::mem::size_of::<u64>()]
                .try_into()
                .unwrap(),
        );
        nonce_as_u64 = nonce_as_u64.wrapping_sub(1);
        self.nonce[0..std::mem::size_of::<u64>()].copy_from_slice(&u64::to_le_bytes(nonce_as_u64));
    }

    pub(crate) fn get(&self) -> &[u8; LOCAL_NONCE_SIZE] {
        &self.nonce
    }
}

#[cfg(test)]
mod tests {
    use super::LocalNonce;

    #[test]
    fn next_works() {
        let mut nonce = LocalNonce::random();
        let nonce_first = nonce.get();
        let nonce_first_as_u64 = u64::from_le_bytes(
            nonce_first[0..std::mem::size_of::<u64>()]
                .try_into()
                .unwrap(),
        );

        nonce.next();
        let nonce_second = nonce.get();
        let nonce_second_as_u64 = u64::from_le_bytes(
            nonce_second[0..std::mem::size_of::<u64>()]
                .try_into()
                .unwrap(),
        );

        assert_eq!(nonce_first_as_u64 + 1, nonce_second_as_u64);
    }

    #[test]
    fn prev_works() {
        let mut nonce = LocalNonce::random();
        let nonce_first = nonce.get();
        let nonce_first_as_u64 = u64::from_le_bytes(
            nonce_first[0..std::mem::size_of::<u64>()]
                .try_into()
                .unwrap(),
        );

        nonce.prev();
        let nonce_second = nonce.get();
        let nonce_second_as_u64 = u64::from_le_bytes(
            nonce_second[0..std::mem::size_of::<u64>()]
                .try_into()
                .unwrap(),
        );

        assert_eq!(nonce_first_as_u64 - 1, nonce_second_as_u64);
    }

    #[test]
    fn next_prev_idempotent() {
        let mut nonce = LocalNonce::random();
        let nonce_first = nonce.get().to_owned();

        nonce.prev();
        nonce.next();

        let nonce_second = nonce.get();
        assert_eq!(&nonce_first, nonce_second);
    }
    #[test]
    fn prev_next_idempotent() {
        let mut nonce = LocalNonce::random();
        let nonce_first = nonce.get().to_owned();

        nonce.next();
        nonce.prev();

        let nonce_second = nonce.get();
        assert_eq!(&nonce_first, nonce_second);
    }
}
