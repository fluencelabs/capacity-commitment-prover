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

pub async fn run_on_all_runnables<'future, Runnable, T, E>(
    runnables: impl Iterator<Item = Runnable>,
    closure: impl Fn(usize, Runnable) -> futures::future::BoxFuture<'future, Result<T, E>>,
) -> Result<Vec<T>, Vec<E>>
where
    T: std::fmt::Debug,
    E: std::fmt::Debug,
{
    use futures::stream::FuturesUnordered;
    use futures::StreamExt;

    let (results, thread_errors): (Vec<_>, Vec<_>) = runnables
        .enumerate()
        .map(|(thread_id, thread)| closure(thread_id, thread))
        .collect::<FuturesUnordered<_>>()
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .partition(Result::is_ok);

    if thread_errors.is_empty() {
        let results = results.into_iter().map(Result::unwrap).collect::<Vec<_>>();

        return Ok(results);
    }

    let thread_errors = thread_errors
        .into_iter()
        .map(Result::unwrap_err)
        .collect::<Vec<_>>();

    Err(thread_errors)
}
