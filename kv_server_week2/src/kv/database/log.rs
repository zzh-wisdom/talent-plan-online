extern crate chrono;
extern crate crossbeam_channel;

use chrono::Local;
use crossbeam_channel::{Receiver, Sender};
use std::collections::BTreeMap;
use std::io;
use std::thread;
use std::thread::JoinHandle;

use super::storage::load;
use super::storage::LogOp;
use super::storage::Storage;
use super::{Key, Value};

/// Time interval for establishing checkpoints, unit seconds
const SET_POINT_INTERVAL: u64 = 3;

/// A Log structure
/// Save the log and set up checkpoints at regular intervals
/// restore the database for database persistence
///
/// Storing logs with threads does not guarantee the security of the data.
/// It is not suitable for scenarios where data security requirements are very strict.
pub struct Log {
    // Channel sender, is used to send logs
    sender: Sender<Option<LogOp>>,
    // Channel receiver, is a pair of sender
    receiver: Option<Receiver<Option<LogOp>>>,
    // Write log thread handle
    handle: Option<JoinHandle<()>>,
    /// File for storing logs
    pub log_file: String,
    /// File for storing checkpoint
    pub check_point_file: String,
    /// File for storing data
    pub data_file: String,

    /// A BTreeMap. Redundant storage of data
    ///
    /// Can prevent the I/O of the database from being blocked when the checkpoint is establishing.
    pub bmap: Option<BTreeMap<Key, Value>>,
}

impl Log {
    /// Constructs a new `Log`.
    ///
    /// Restore data from the file and save it to self.bmap
    ///
    /// In order to avoid repeating the recovery of data from the file,
    /// the self.bmap will be cloned directly into the database.
    ///
    /// Create a channel，save in self.sender and self.Receiver
    pub fn new(log_file: &String, check_point_file: &String, data_file: &String) -> Log {
        let (tx, rx) = crossbeam_channel::bounded(1);
        let mut bmap = BTreeMap::new();
        match load(&mut bmap, data_file, check_point_file, log_file) {
            Ok(_) => {}
            Err(e) => println!("{}", e.to_string()),
        };
        Log {
            sender: tx,
            receiver: Some(rx),
            handle: None,
            log_file: log_file.clone(),
            check_point_file: check_point_file.clone(),
            data_file: data_file.clone(),
            bmap: Some(bmap),
        }
    }

    /// Write logs.
    /// In essence, send a log to the channel, waiting for the log thread to write
    ///
    /// #Examples
    ///
    /// ```
    /// use kv_server_week2::database::log::Log;
    /// use kv_server_week2::database::storage::LogOp;
    /// let mut log = Log::new(&"log_file.dat".to_string(),
    ///           &"check_point_file.dat".to_string(), &"data_file.dat".to_string());
    /// log.start();
    /// log.record(Some(LogOp::OpClear));
    /// ```
    pub fn record(&self, log_op: Option<LogOp>) {
        self.sender.send(log_op);
    }

    /// Create a write log thread
    ///
    /// When the write log thread is established firstly, Option will be taked from self.receiver.
    /// Option will also be taked from self.bmap.
    /// When it is detected that the data in self.receiver is None, the establishment of the write log thread fails.
    /// In essence, only one write log thread can be created, that is, `start` function is only valid for the first call.
    pub fn start(&mut self) -> Result<(), io::Error> {
        println!("starting log thread...");
        if self.receiver.is_none() {
            println!("Log thread has been started.");
            return Ok(());
        }
        let rx = self.receiver.take().unwrap();
        let log_file = self.log_file.clone();
        let check_point_file = self.check_point_file.clone();
        let data_file = self.data_file.clone();
        if self.bmap.is_none() {
            // Normally this will not happen.
            println!("log start error, bmap none!");
            return Ok(());
        }
        let bmap = self.bmap.take().unwrap();

        let handle = thread::Builder::new()
            .name("log thread".to_string())
            .spawn(move || write_log(bmap, rx, log_file, check_point_file, data_file))?;
        self.handle = Some(handle);
        println!("starting log thread success.");
        Ok(())
    }

    /// End writing log thread
    pub fn stop(&mut self) {
        if self.handle.is_none() {
            return;
        }
        let handle = self.handle.take().unwrap();
        self.sender.send(None);
        // Ensure thread safe exit
        if handle.join().is_err() {
            println!("Log close error.");
        };
    }
}

// The function executes in the write-log thread
fn write_log(
    mut bmap: BTreeMap<Key, Value>,
    rx: Receiver<Option<LogOp>>,
    log_file: String,
    check_point_file: String,
    data_file: String,
) {
    let mut store =
        Storage::open(&data_file, &log_file, &check_point_file).expect("Storage open failed.");
    // Use the time difference between end_time and start_time to determine if a checkpoint needs to be established
    let mut start_time = Local::now().timestamp_millis();
    let mut end_time;

    // Flags whether there is a log with no checkpoint established
    let mut is_empty = true;
    loop {
        let log_op = match rx.try_recv() {
            Some(msg) => match msg {
                Some(log_op) => Some(log_op),
                None => break,
            },
            None => None,
        };
        if log_op.is_some() {
            is_empty = false;
            let log_op = log_op.unwrap();
            match store.save_log(&log_op) {
                Ok(_) => {}
                Err(e) => println!("store save_log error.Error:{}", e.to_string()),
            }
            // For each log, execute it once on the redundant tree to keep the data in sync
            match log_op {
                LogOp::OpDel(key) => bmap.remove(&key),
                LogOp::OpSet(key, value) => bmap.insert(key.to_string(), value.to_string()),
                LogOp::OpClear => {
                    bmap.clear();
                    None
                }
            };
        }
        // When `is_empty` is true, there is no new log, even if the time is up, no new checkpoint will be created.
        if !is_empty {
            end_time = Local::now().timestamp_millis();
            if ((end_time - start_time) as f64) / 1000.0 >= SET_POINT_INTERVAL as f64 {
                println!("Time Point {} ...", (end_time as f64) / 1000.0);
                println!("Set Point...");
                let bmap = bmap.clone();
                store.establish_check_point(bmap);
                start_time = end_time;
                is_empty = true;
            }
        }
    }
    store.close();
    println!("write_log thread end!");
}
