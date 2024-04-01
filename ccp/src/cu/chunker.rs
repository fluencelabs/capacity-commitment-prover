pub(crate) struct Chunker {
    // inclusive
    start: u64,
    chunk_size: u64,
    // exclusive
    end: u64,
}

impl Chunker {
    pub(crate) fn new(size: u64, chunk_size: u64) -> Self {
        Self {
            start: 0,
            chunk_size,
            end: size,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Chunk {
    pub(crate) start: u64,
    pub(crate) size: u64,
}

impl Iterator for Chunker {
    type Item = Chunk;

    fn next(&mut self) -> Option<Self::Item> {
        // we add 1 to substracted and 1 to result because we want to have None when end == start, not Some(0).
        let remains = self.end.checked_sub(self.start + 1).map(|n| n + 1);

        remains.map(|remains| {
            let capped_chunk_size = std::cmp::min(remains, self.chunk_size);
            let chunk = Chunk {
                start: self.start,
                size: capped_chunk_size,
            };
            self.start += capped_chunk_size;
            chunk
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_divisible() {
        let chunker = Chunker::new(100, 50);
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
        let chunker = Chunker::new(50, 50);
        let vals: Vec<_> = chunker.collect();
        assert_eq!(vals, vec![Chunk { start: 0, size: 50 }]);
    }

    #[test]
    fn test_uneven() {
        let chunker = Chunker::new(120, 50);
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
        let chunker = Chunker::new(20, 50);
        let vals: Vec<_> = chunker.collect();
        assert_eq!(vals, vec![Chunk { start: 0, size: 20 }]);
    }
}
