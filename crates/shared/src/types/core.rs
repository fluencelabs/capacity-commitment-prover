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

use newtype_derive::newtype_fmt;
use serde::Deserialize;
use serde::Serialize;

pub type CPUIdType = u32;

/// An opaque type that represents a CPU physical core.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
#[repr(transparent)]
pub struct PhysicalCoreId(CPUIdType);

/// An opaque type that represents a CPU logical core.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
#[repr(transparent)]
pub struct LogicalCoreId(CPUIdType);

newtype_derive::NewtypeFrom! { () pub struct PhysicalCoreId(CPUIdType); }
newtype_derive::NewtypeDisplay! { () pub struct PhysicalCoreId(CPUIdType); }

newtype_derive::NewtypeFrom! { () pub struct LogicalCoreId(CPUIdType); }
newtype_derive::NewtypeDisplay! { () pub struct LogicalCoreId(CPUIdType); }

impl PhysicalCoreId {
    pub const fn new(core_id: CPUIdType) -> Self {
        Self(core_id)
    }
}

impl LogicalCoreId {
    pub const fn new(core_id: CPUIdType) -> Self {
        Self(core_id)
    }
}

impl From<PhysicalCoreId> for usize {
    fn from(value: PhysicalCoreId) -> usize {
        value.0 as usize
    }
}

impl From<LogicalCoreId> for usize {
    fn from(value: LogicalCoreId) -> usize {
        value.0 as usize
    }
}
