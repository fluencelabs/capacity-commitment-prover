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

/// Runs the provided closure on all supplied runnables concurrently in the unordered way.
pub async fn run_unordered<'future, Runnable, T, E>(
    runnables: impl Iterator<Item = Runnable>,
    closure: impl Fn(usize, Runnable) -> futures::future::BoxFuture<'future, Result<T, E>>,
) -> Result<Vec<T>, Vec<E>>
where
    T: std::fmt::Debug,
    E: std::fmt::Debug,
{
    use futures::stream::FuturesUnordered;
    use futures::StreamExt;

    let (results, errors) = runnables
        .enumerate()
        .map(|(idx, runnable)| closure(idx, runnable))
        .collect::<FuturesUnordered<_>>()
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .partition::<Vec<_>, _>(Result::is_ok);

    if errors.is_empty() {
        let results = unwrap(results.into_iter(), Result::unwrap);
        return Ok(results);
    }

    let errors = unwrap(errors.into_iter(), Result::unwrap_err);
    Err(errors)
}

fn unwrap<W, U>(wrapped_values: impl Iterator<Item = W>, unwrapper: impl FnMut(W) -> U) -> Vec<U> {
    wrapped_values.map(unwrapper).collect::<Vec<_>>()
}
