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

use ccp_shared::types::LocalNonce;

pub(crate) trait NonceIterable {
    /// Generates the next nonce.
    fn next(&mut self);

    /// Returns back to the previous nonce/
    fn prev(&mut self);
}

impl NonceIterable for LocalNonce {
    fn next(&mut self) {
        let mut nonce_as_u64: u64 = u64::from_le_bytes(
            self.as_mut()[0..std::mem::size_of::<u64>()]
                .try_into()
                .unwrap(),
        );
        nonce_as_u64 = nonce_as_u64.wrapping_add(1);
        self.as_mut()[0..std::mem::size_of::<u64>()]
            .copy_from_slice(&u64::to_le_bytes(nonce_as_u64));
    }

    fn prev(&mut self) {
        let mut nonce_as_u64: u64 = u64::from_le_bytes(
            self.as_mut()[0..std::mem::size_of::<u64>()]
                .try_into()
                .unwrap(),
        );
        nonce_as_u64 = nonce_as_u64.wrapping_sub(1);
        self.as_mut()[0..std::mem::size_of::<u64>()]
            .copy_from_slice(&u64::to_le_bytes(nonce_as_u64));
    }
}

#[cfg(test)]
mod tests {
    use super::LocalNonce;
    use super::NonceIterable;

    #[test]
    fn next_works() {
        let mut nonce = LocalNonce::random();
        let nonce_first = nonce.as_ref();
        let nonce_first_as_u64 = u64::from_le_bytes(
            nonce_first[0..std::mem::size_of::<u64>()]
                .try_into()
                .unwrap(),
        );

        nonce.next();
        let nonce_second = nonce.as_ref();
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
        let nonce_first = nonce.as_ref();
        let nonce_first_as_u64 = u64::from_le_bytes(
            nonce_first[0..std::mem::size_of::<u64>()]
                .try_into()
                .unwrap(),
        );

        nonce.prev();
        let nonce_second = nonce.as_ref();
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
        let nonce_first = nonce.as_ref().to_owned();

        nonce.prev();
        assert_ne!(&nonce_first, nonce.as_ref());
        nonce.next();

        let nonce_second = nonce.as_ref();
        assert_eq!(&nonce_first, nonce_second);
    }

    #[test]
    fn prev_next_idempotent() {
        let mut nonce = LocalNonce::random();
        let nonce_first = nonce.as_ref().to_owned();

        nonce.next();
        assert_ne!(&nonce_first, nonce.as_ref());
        nonce.prev();

        let nonce_second = nonce.as_ref();
        assert_eq!(&nonce_first, nonce_second);
    }
}
