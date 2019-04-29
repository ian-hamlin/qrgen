use csv;
use log::{trace, warn};
use std::io::Read;

pub struct Chunker<T>
where
    T: Read,
{
    inner: csv::Reader<T>,
    chunk_size: usize,
}

impl<T: Read> Chunker<T> {
    pub fn new(reader: csv::Reader<T>, chunk_size: usize) -> Self {
        Chunker {
            inner: reader,
            chunk_size,
        }
    }
}

impl<T: Read> Iterator for Chunker<T> {
    type Item = Vec<csv::StringRecord>;

    fn next(&mut self) -> Option<Vec<csv::StringRecord>> {
        trace!("iterator for Chunker<T>");
        let mut chunks = Vec::with_capacity(self.chunk_size);

        for (total, result) in self.inner.records().enumerate() {
            match result {
                Ok(r) => chunks.push(r),
                Err(e) => warn!("{:?}", e),
            }

            // Exit reading at this stage if we reached the chunk size.
            if total == self.chunk_size - 1 {
                break;
            }
        }

        if chunks.is_empty() {
            return None;
        }

        Some(chunks)
    }
}
