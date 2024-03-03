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

use config::Config;
use config::File;
use config::FileFormat;

use crate::unresolved_config::UnresolvedCCPConfig;
use crate::CCPConfig;

pub fn load_config(path: &str) -> eyre::Result<CCPConfig> {
    let config_source = File::with_name(path)
        .required(true)
        .format(FileFormat::Toml);
    let config = Config::builder().add_source(config_source).build()?;

    let config: UnresolvedCCPConfig = config.try_deserialize()?;
    config.resolve()
}
