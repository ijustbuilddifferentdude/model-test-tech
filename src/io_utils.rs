use anyhow::Result;
use flate2::Compression;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Write};

pub fn open_csv_reader(path: &str) -> Result<Box<dyn Read>> {
    let file = File::open(path)?;
    let rdr: Box<dyn Read> = if path.ends_with(".gz") {
        Box::new(GzDecoder::new(file))
    } else {
        Box::new(file)
    };
    Ok(rdr)
}

pub fn open_csv_writer(path: &str) -> Result<Box<dyn Write>> {
    let file = File::create(path)?;
    let w: Box<dyn Write> = if path.ends_with(".gz") {
        Box::new(GzEncoder::new(file, Compression::new(6)))
    } else {
        Box::new(file)
    };
    Ok(w)
}

pub fn buf_read(r: Box<dyn Read>) -> BufReader<Box<dyn Read>> {
    BufReader::with_capacity(1 << 20, r)
}

pub fn buf_write(w: Box<dyn Write>) -> BufWriter<Box<dyn Write>> {
    BufWriter::with_capacity(1 << 20, w)
}
