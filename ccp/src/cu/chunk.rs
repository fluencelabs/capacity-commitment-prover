/*
 * Copyright 2024 Fluence DAO
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

pub(crate) struct ChunkGenerator {
    // inclusive
    next_chunk_start: u64,
    chunk_size: u64,
    // exclusive
    range_end: u64,
}

impl ChunkGenerator {
    pub(crate) fn new(range_start: u64, range_end: u64, chunk_size: u64) -> Self {
        Self {
            next_chunk_start: range_start,
            chunk_size,
            range_end,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Chunk {
    pub(crate) start: u64,
    pub(crate) size: u64,
}

impl Iterator for ChunkGenerator {
    type Item = Chunk;

    fn next(&mut self) -> Option<Self::Item> {
        if self.range_end <= self.next_chunk_start {
            return None;
        }

        let remains = self.range_end - self.next_chunk_start;

        let capped_chunk_size = remains.clamp(0, self.chunk_size);
        let chunk = Chunk {
            start: self.next_chunk_start,
            size: capped_chunk_size,
        };
        self.next_chunk_start += capped_chunk_size;
        Some(chunk)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_divisible() {
        let chunker = ChunkGenerator::new(0, 100, 50);
        let vals: Vec<_> = chunker.collect();
        assert_eq!(
            vals,
            vec![
                Chunk { start: 0, size: 50 },
                Chunk {
                    start: 50,
                    size: 50,
                }
            ]
        );
    }

    #[test]
    fn test_divisible_single() {
        let chunker = ChunkGenerator::new(0, 50, 50);
        let vals: Vec<_> = chunker.collect();
        assert_eq!(vals, vec![Chunk { start: 0, size: 50 }]);
    }

    #[test]
    fn test_uneven() {
        let chunker = ChunkGenerator::new(0, 120, 50);
        let vals: Vec<_> = chunker.collect();
        assert_eq!(
            vals,
            vec![
                Chunk { start: 0, size: 50 },
                Chunk {
                    start: 50,
                    size: 50,
                },
                Chunk {
                    start: 100,
                    size: 20,
                }
            ]
        );
    }

    #[test]
    fn test_uneven_single() {
        let chunker = ChunkGenerator::new(0, 20, 50);
        let vals: Vec<_> = chunker.collect();
        assert_eq!(vals, vec![Chunk { start: 0, size: 20 }]);
    }

    #[test]
    fn test_start() {
        let chunker = ChunkGenerator::new(10, 25, 11);
        let vals: Vec<_> = chunker.collect();
        assert_eq!(
            vals,
            vec![
                Chunk {
                    start: 10,
                    size: 11
                },
                Chunk { start: 21, size: 4 },
            ]
        );
    }
}
