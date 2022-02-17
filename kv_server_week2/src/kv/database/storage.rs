extern crate crossbeam_channel;
extern crate serde;
extern crate serde_json;

use crossbeam_channel::{Receiver, Sender};
use serde::{Deserialize, Serialize};
use serde_json::Deserializer;

use std::collections::BTreeMap;
use std::fs::{File, OpenOptions};
use std::io::BufRead;
use std::io::{self, BufReader, BufWriter, Read, Seek, SeekFrom, Write};
use std::thread;
use std::thread::JoinHandle;

use super::error::Result;
use super::{Key, Value};

use std::string::ToString;

/// Log format
#[derive(Serialize, Deserialize, Debug)]
pub enum LogOp {
    OpDel(Key),
    OpSet(Key, Value),
    OpClear,
}

/// Save key-value pairs
#[derive(Serialize, Deserialize, Debug)]
struct KeyValue {
    key: Key,
    value: Value,
}

impl KeyValue {
    /// Generate a KeyValue using string
    fn from(key: &String, value: &String) -> Self {
        KeyValue {
            key: key.to_string(),
            value: value.to_string(),
        }
    }
}

/// structure Storage
///
/// Dealing with documents
/// Save logs, checkpoints, and data
pub struct Storage {
    // Channel sender, sending data that needs to be backed up
    sender: Sender<Option<BTreeMap<Key, Value>>>,
    // Backup data thread handle
    handle: Option<JoinHandle<()>>,

    // Write log
    log_writer: BufWriterWithPos<File>,
    // Write checkpoint
    check_point_writer: BufWriter<File>,
}

impl Storage {
    /// Opens a `Storage` with the given files.
    ///
    /// This will create a new directory if the given one does not exist.
    /// Every time openning the log file and checkpoint file, the data in the file will be cleared to prevent the file from being too large.
    ///
    /// # Errors
    ///
    /// It propagates I/O or deserialization errors when the file open and data write.
    pub fn open(
        data_file: &String,
        log_file: &String,
        check_point_file: &String,
    ) -> Result<Storage> {
        let log_writer = new_bufwriter_with_pos(log_file)?;
        let check_point_writer = new_bufwriter(check_point_file)?;

        let (tx, rx) = crossbeam_channel::bounded(1);
        let data_file = data_file.to_string();
        let handle = thread::Builder::new()
            .name("Copy data thread".to_string())
            .spawn(move || copy_data(data_file, rx))?;

        Ok(Storage {
            sender: tx,
            handle: Some(handle),
            log_writer,
            check_point_writer,
        })
    }

    /// Save logs
    ///
    /// # Errors
    ///
    /// It propagates I/O or deserialization errors when data write.
    pub fn save_log(&mut self, log: &LogOp) -> Result<()> {
        serde_json::to_writer(&mut self.log_writer, &log)?;
        self.log_writer.flush()?;
        Ok(())
    }

    /// Establish checkpoint
    ///
    /// Write the offset of the log file to the checkpoint file
    /// Send a backup of the data to the channel
    pub fn establish_check_point(&mut self, bmap: BTreeMap<Key, Value>) {
        self.check_point_writer
            .write_fmt(format_args!("{}\n", self.log_writer.pos))
            .expect("check_point_writer error");
        self.check_point_writer
            .flush()
            .expect("point_writer flush error");
        self.sender.send(Some(bmap));
    }

    /// Close Storage
    ///
    /// Need to wait for the backup data thread to end
    pub fn close(&mut self) {
        if self.handle.is_none() {
            return;
        }
        let handle = self.handle.take().unwrap();
        self.sender.send(None);
        if handle.join().is_err() {
            println!("Storage close error.")
        };
    }
}

/// The function executes in the backup-data thread
fn copy_data(data_file: String, rx: Receiver<Option<BTreeMap<Key, Value>>>) {
    loop {
        match rx.recv() {
            Some(bmap) => {
                if let Some(bmap) = bmap {
                    match save_date(&data_file, bmap) {
                        Ok(_) => {}
                        Err(s) => println!("Error:{}", s.to_string()),
                    };
                } else {
                    break;
                }
            }
            None => continue,
        }
    }
}

/// Save data to the file
///
/// # Errors
///
/// It propagates I/O or deserialization errors when data write.
fn save_date(data_file: &String, bmap: BTreeMap<Key, Value>) -> Result<()> {
    println!("Save data...");
    let mut data_writer = new_bufwriter_with_pos(data_file)?;
    for (k, v) in bmap {
        let data = KeyValue::from(&k, &v);
        serde_json::to_writer(&mut data_writer, &data)?;
    }
    data_writer.flush()?;
    println!("Save over!");
    Ok(())
}

/// Recover data from files
///
/// # Errors
///
/// It propagates I/O or deserialization errors when data read.
pub fn load(
    bmap: &mut BTreeMap<Key, Value>,
    data_file: &String,
    check_point_file: &String,
    log_file: &String,
) -> Result<()> {
    // Load data from data file
    println!("Start BtreeMap {} load...", data_file);
    let data_file_reader = new_bufreader_with_pos(data_file);
    match data_file_reader {
        Err(e) => println!(
            "Data file:\"{}\" not exist!\nError:{}",
            data_file,
            e.to_string()
        ),
        Ok(reader) => {
            let mut stream = Deserializer::from_reader(reader).into_iter::<KeyValue>();
            while let Some(data) = stream.next() {
                let data = data?;
                bmap.insert(data.key, data.value);
            }
        }
    }
    println!("BtreeMap {} load over!", data_file);

    // Find the latest checkpoint from the checkpoint file
    // The default is 0
    let mut check_point: u64 = 0;
    let chech_point_reader = new_bufreader(check_point_file);
    match chech_point_reader {
        Err(e) => println!(
            "Check point file:\"{}\" not exist!\nError:{}",
            check_point_file,
            e.to_string()
        ),
        Ok(reader) => {
            let mut it = reader.lines();
            let mut last_line = it.next();
            let mut line = it.next();
            while line.is_some() {
                last_line = line;
                line = it.next();
            }
            if last_line.is_some() {
                let s = last_line.unwrap().unwrap();
                check_point = match s.trim().parse() {
                    Ok(num) => num,
                    Err(e) => {
                        println!("Error:{}", e.to_string());
                        0
                    }
                };
            }
        }
    }
    println!("Check piont: {}", check_point);

    // Re-execute the log after the checkpoint
    println!("Start do Log {} ...", log_file);
    let log_file_reader = new_bufreader_with_pos(log_file);
    match log_file_reader {
        Err(e) => println!(
            "Log file:\"{}\" not exist!\nError:{}",
            log_file,
            e.to_string()
        ),
        Ok(reader) => {
            let mut stream = Deserializer::from_reader(reader).into_iter::<LogOp>();
            while let Some(log) = stream.next() {
                match log? {
                    LogOp::OpDel(key) => bmap.remove(&key),
                    LogOp::OpSet(key, value) => bmap.insert(key.to_string(), value.to_string()),
                    LogOp::OpClear => {
                        bmap.clear();
                        None
                    }
                };
            }
        }
    }
    println!("Do log over!");
    Ok(())
}

/// Create a BufReader with the specified file
///
/// Returns the BufReader to the file.
fn new_bufreader(file: &String) -> Result<BufReader<File>> {
    let reader = BufReader::new(OpenOptions::new().read(true).open(file)?);
    Ok(reader)
}

/// Create a BufReaderWithPos with the specified file
///
/// Returns the BufReaderWithPos to the file.
fn new_bufreader_with_pos(file: &String) -> Result<BufReaderWithPos<File>> {
    let reader = BufReaderWithPos::new(OpenOptions::new().read(true).open(file)?)?;
    Ok(reader)
}

/// Create a BufWriter with the specified file
///
/// Returns the BufWriter to the file.
fn new_bufwriter(file: &String) -> Result<BufWriter<File>> {
    let writer = BufWriter::new(
        OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(file)?,
    );
    Ok(writer)
}

/// Create a BufWriterWithPos with the specified file
///
/// Returns the BufWriterWithPos to the file.
fn new_bufwriter_with_pos(file: &String) -> Result<BufWriterWithPos<File>> {
    let writer = BufWriterWithPos::new(
        OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(file)?,
    )?;
    Ok(writer)
}

/// Similar to BufReader, but save the current offset
struct BufReaderWithPos<R: Read + Seek> {
    reader: BufReader<R>,
    pos: u64,
}

impl<R: Read + Seek> BufReaderWithPos<R> {
    fn new(mut inner: R) -> Result<Self> {
        let pos = inner.seek(SeekFrom::Current(0))?;
        Ok(BufReaderWithPos {
            reader: BufReader::new(inner),
            pos,
        })
    }
}

impl<R: Read + Seek> Read for BufReaderWithPos<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.reader.read(buf)
    }
}

impl<R: Read + Seek> Seek for BufReaderWithPos<R> {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        self.pos = self.reader.seek(pos)?;
        Ok(self.pos)
    }
}

/// Similar to BufWriter, but save the current offset
struct BufWriterWithPos<W: Write + Seek> {
    writer: BufWriter<W>,
    pos: u64,
}

impl<W: Write + Seek> BufWriterWithPos<W> {
    fn new(mut inner: W) -> Result<Self> {
        let pos = inner.seek(SeekFrom::Current(0))?;
        Ok(BufWriterWithPos {
            writer: BufWriter::new(inner),
            pos,
        })
    }
}

impl<W: Write + Seek> Write for BufWriterWithPos<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let len = self.writer.write(buf)?;
        self.pos += len as u64;
        Ok(len)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.writer.flush()
    }
}

impl<W: Write + Seek> Seek for BufWriterWithPos<W> {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        self.pos = self.writer.seek(pos)?;
        Ok(self.pos)
    }
}
