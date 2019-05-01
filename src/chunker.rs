use csv;
use log::warn;
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
        let mut chunks = Vec::with_capacity(self.chunk_size);

        for (total, result) in self.inner.records().enumerate() {
            match result {
                Ok(r) => {
                    chunks.push(r);
                }
                Err(e) => warn!("{:?}", e),
            }

            // Exit reading at this stage if we reached the chunk size.
            if total == self.chunk_size - 1 {
                break;
            }
        }

        // This assumes that at least 1 line in the chunk was valid.
        // TODO - do something about this, what if the first chunk is all wrong?
        if chunks.is_empty() {
            return None;
        }

        Some(chunks)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_give_chunks() {
        let input = "12\n34\n56\n78\n90".as_bytes();
        let reader = csv::ReaderBuilder::new()
            .has_headers(false)
            .from_reader(input);
        let mut chunks = Chunker::new(reader, 2);

        let chunk = chunks.next().unwrap();
        assert_eq!(chunk.len(), 2);
        assert_eq!(csv::StringRecord::from(vec!["12"]), chunk[0]);
        assert_eq!(csv::StringRecord::from(vec!["34"]), chunk[1]);

        let chunk = chunks.next().unwrap();
        assert_eq!(chunk.len(), 2);
        assert_eq!(csv::StringRecord::from(vec!["56"]), chunk[0]);
        assert_eq!(csv::StringRecord::from(vec!["78"]), chunk[1]);

        let chunk = chunks.next().unwrap();
        assert_eq!(chunk.len(), 1);
        assert_eq!(csv::StringRecord::from(vec!["90"]), chunk[0]);

        assert_eq!(None, chunks.next());
    }
}
