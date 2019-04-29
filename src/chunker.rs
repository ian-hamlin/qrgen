use csv;
use std::{error::Error, fs::File, io::Read, path::PathBuf};

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
