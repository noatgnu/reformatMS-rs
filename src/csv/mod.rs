use std::path::PathBuf;
use std::fs;
use std::io::{BufRead, BufReader, Result};
use std::str;
use std::fs::File;

pub struct CSVFile {
    pub buffer: Vec<u8>,
    pub reader: BufReader<File>,
}

impl Iterator for CSVFile {
    type Item = String;
    fn next(&mut self) -> Option<String> {
        match self.reader.read_until(b'\n', &mut self.buffer) {
            Err(error) => panic!("Can't read: {}", error),
            Ok(0) => None,
            Ok(result) => Some(str::from_utf8(&self.buffer).unwrap().to_string()),
        }
    }
}

pub fn read_csv(file_path: &PathBuf) -> CSVFile {
    let file = match File::open(file_path) {
        Err(error) => panic!("Can't open {:?}: {}", file_path, error),
        Ok(result) => result,
    };

    return CSVFile{
        buffer: vec![],
        reader: BufReader::new(file),
    }
}