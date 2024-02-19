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

use crate::types::Difficulty;
use crate::types::ResultHash;

pub trait MeetDifficulty {
    fn meet_difficulty(&self, difficulty: &Difficulty) -> bool;
}

impl MeetDifficulty for ResultHash {
    fn meet_difficulty(&self, difficulty: &Difficulty) -> bool {
        self.as_ref() < difficulty.as_ref()
    }
}
