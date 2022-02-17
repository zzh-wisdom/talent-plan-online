use std::collections::{BTreeMap, HashMap};
use std::sync::{Arc, Mutex, RwLock};

use super::engine_trait::Engine;
use super::log::Log;
use super::storage::LogOp;
use super::{Key, Value};

// File for storing logs
pub const DEFAULT_LOG_FILE: &'static str = "default_log.log";
// File for storing checkpoint
pub const DEFAULT_CHECK_POINT_FILE: &'static str = "default_check_point.txt";
// File for storing data
pub const DEFAULT_DATA_FILE: &'static str = "default_data.data";

/// A database engine.
/// The container uses BTreeMap
#[derive(Clone)]
pub struct BtreeMapEngine {
    //Thread safety with reference counting and locks
    db: Arc<RwLock<BTreeMap<Key, Value>>>,
    log: Arc<Mutex<Log>>,
}

impl BtreeMapEngine {
    /// Constructs a new `BtreeMapEngine`.
    /// Can automatically recover data from files.
    pub fn new() -> Self {
        let mut log = Log::new(
            &DEFAULT_LOG_FILE.to_string(),
            &DEFAULT_CHECK_POINT_FILE.to_string(),
            &DEFAULT_DATA_FILE.to_string(),
        );
        let bmap = log.bmap.clone();
        let bmap = match bmap {
            Some(bmap) => bmap,
            // Normally this will not happen.
            None => panic!("new BtreeMapEngine error!"),
        };
        log.start().ok();
        let engine = BtreeMapEngine {
            db: Arc::new(RwLock::new(bmap)),
            log: Arc::new(Mutex::new(log)),
        };
        engine
    }

    /// Close the database engine
    pub fn shutdown(&mut self) {
        println!("BtreeMap engine shutdown...");
        match self.log.lock() {
            Ok(mut log) => {
                log.stop();
            }
            Err(e) => {
                println!("BtreeMapEngine log lock error.\nError:{}", e.to_string());
            }
        }
    }
}

/// Implement the trait of Engine for BtreeMapEngine
impl Engine for BtreeMapEngine {
    /// The operations which change the data in database need to save the log first.
    ///
    /// #Panics
    ///
    /// Panics if there is database's read-write lock exception
    fn get(&self, key: Key) -> Result<Option<Value>, ()> {
        let db = match self.db.read() {
            Ok(db) => db,
            Err(e) => panic!("read lock failed, {}", e.to_string()),
        };
        let ret = db.get(&key);
        match ret {
            Some(value) => Ok(Some(value.clone())),
            None => Ok(None),
        }
    }
    fn set(&self, key: Key, value: Value) -> Result<Option<Value>, ()> {
        // write log first.
        match self.log.lock() {
            Ok(log) => {
                log.record(Some(LogOp::OpSet(key.clone(), value.clone())));
            }
            Err(e) => {
                println!("BtreeMapEngine log lock error.\nError:{}", e.to_string());
            }
        }
        let mut db = match self.db.write() {
            Ok(db) => db,
            Err(e) => panic!("write lock failed, {}", e.to_string()),
        };
        let ret = db.insert(key.clone(), value.clone());
        match ret {
            //the key exists.
            Some(value) => Ok(Some(value.clone())),
            None => Ok(None),
        }
    }
    fn insert(&self, key: Key, value: Value) -> Result<bool, ()> {
        let mut db = match self.db.write() {
            Ok(db) => db,
            Err(e) => panic!("write lock failed, {}", e.to_string()),
        };
        if !db.contains_key(&key) {
            // Only when the key exists, can the key-value pair insert.
            match self.log.lock() {
                Ok(log) => {
                    log.record(Some(LogOp::OpSet(key.clone(), value.clone())));
                }
                Err(e) => {
                    println!("BtreeMapEngine log lock error.\nError:{}", e.to_string());
                }
            }

            db.insert(key.clone(), value.clone());
            Ok(true)
        } else {
            Ok(false)
        }
    }
    fn update(&self, key: Key, value: Value) -> Result<Option<Value>, ()> {
        let mut db = match self.db.write() {
            Ok(db) => db,
            Err(e) => panic!("write lock failed, {}", e.to_string()),
        };
        if let Some(x) = db.get_mut(&key) {
            // Only when the key exists, can the key-value pair update.
            match self.log.lock() {
                Ok(log) => {
                    log.record(Some(LogOp::OpSet(key.clone(), value.clone())));
                }
                Err(e) => {
                    println!("BtreeMapEngine log lock error.\nError:{}", e.to_string());
                }
            }

            let v = (*x).to_string();
            *x = value;
            Ok(Some(v))
        } else {
            Ok(None)
        }
    }
    fn delete(&self, key: Key) -> Result<Option<Value>, ()> {
        match self.log.lock() {
            Ok(log) => {
                log.record(Some(LogOp::OpDel(key.clone())));
            }
            Err(e) => {
                println!("BtreeMapEngine log lock error.\nError:{}", e.to_string());
            }
        }
        let mut db = match self.db.write() {
            Ok(db) => db,
            Err(e) => panic!("write lock failed, {}", e.to_string()),
        };
        let ret = db.remove(&key);
        match ret {
            // Delete existing key-value pair, and return deleted value
            Some(value) => Ok(Some(value)),
            // Delete not existing key-value pair, and return none
            None => Ok(None),
        }
    }
    fn scan(&self, key_start: Key, key_end: Key) -> Result<Option<HashMap<Key, Value>>, ()> {
        let db = match self.db.read() {
            Ok(db) => db,
            Err(e) => panic!("read lock failed, {}", e.to_string()),
        };
        let mut hmap = HashMap::new();
        for (k, v) in db.range(key_start.clone()..key_end.clone()) {
            hmap.insert(k.clone(), v.clone());
        }
        // Returns Ok(None) when the database is empty
        if hmap.len() != 0 {
            Ok(Some(hmap))
        } else {
            Ok(None)
        }
    }
    fn scan_all(&self) -> Result<Option<HashMap<Key, Value>>, ()> {
        let mut hmap = HashMap::new();
        let db = match self.db.read() {
            Ok(db) => db,
            Err(e) => panic!("read lock failed, {}", e.to_string()),
        };
        for (k, v) in &*db {
            hmap.insert(k.clone(), v.clone());
        }
        if hmap.len() != 0 {
            Ok(Some(hmap))
        } else {
            Ok(None)
        }
    }

    fn clear(&self) -> Result<bool, ()> {
        match self.log.lock() {
            Ok(log) => {
                log.record(Some(LogOp::OpClear));
            }
            Err(e) => {
                println!("BtreeMapEngine log lock error.\nError:{}", e.to_string());
            }
        }
        let mut db = match self.db.write() {
            Ok(db) => db,
            Err(e) => panic!("write lock failed, {}", e.to_string()),
        };
        db.clear();
        Ok(true)
    }
}
