use csv::Reader;
use serde::de::DeserializeOwned;
use std::{fs::File, io::BufReader};

pub fn read_csv_from_file<T: DeserializeOwned>(file_path: &str) -> Result<Vec<T>, csv::Error> {
    let mut reader = csv::ReaderBuilder::new()
        .trim(csv::Trim::All)
        .flexible(true)
        .from_path(file_path)?;
    reader.deserialize().collect::<Result<Vec<T>, csv::Error>>()
}

pub fn csv_buf_reader_for_file(file_path: &str) -> Result<Reader<BufReader<File>>, csv::Error> {
    let file = BufReader::new(std::fs::File::open(file_path)?);
    Ok(csv::ReaderBuilder::new()
        .trim(csv::Trim::All)
        .flexible(true)
        .from_reader(file))
}

pub fn read_csv_from_buffer<T: DeserializeOwned>(buf: &Vec<u8>) -> Result<Vec<T>, csv::Error> {
    let mut reader = csv::ReaderBuilder::new()
        .trim(csv::Trim::All)
        .flexible(true)
        .from_reader(buf.as_slice());
    reader.deserialize().collect::<Result<Vec<T>, csv::Error>>()
}
