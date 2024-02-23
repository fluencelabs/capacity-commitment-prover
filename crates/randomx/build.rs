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

use std::env;
use std::path::Path;
use std::path::PathBuf;

use cmake::Config;

const RANDOMX_PATH: &str = "randomx";

fn main() {
    let randomx_path = build_randomx(RANDOMX_PATH);
    link_randomx(randomx_path);

    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap_or("linux".to_string());
    link_cpp_runtime(&target_os);
}

fn build_randomx(randomx_path: impl AsRef<Path>) -> PathBuf {
    Config::new(randomx_path).define("DARCH", "native").build()
}

fn link_randomx(randomx_path: PathBuf) {
    println!(
        "cargo:rustc-link-search=native={}/lib",
        randomx_path.display()
    );
    println!("cargo:rustc-link-lib=static=randomx");
}

fn link_cpp_runtime(target_os: &str) {
    let dylib_name = match target_os {
        "macos" | "ios" => "c++",
        _ => "stdc++",
    };

    println!("cargo:rustc-link-lib=dylib={}", dylib_name);
}
