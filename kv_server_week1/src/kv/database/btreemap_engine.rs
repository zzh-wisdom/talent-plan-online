use std::collections::{BTreeMap, HashMap};
use std::sync::{Arc, RwLock};

use super::engine_trait::Engine;
use super::{Key, Value};

/// A database engine.
/// The container uses BTreeMap
#[derive(Clone)]
pub struct BtreeMapEngine {
    //Thread safety with reference counting and locks
    db: Arc<RwLock<BTreeMap<Key, Value>>>,
}

impl BtreeMapEngine {
    /// Constructs a new, empty `BtreeMapEngine`.
    pub fn new() -> Self {
        BtreeMapEngine {
            db: Arc::new(RwLock::new(BTreeMap::new())),
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
            let v = (*x).to_string();
            *x = value;
            Ok(Some(v))
        } else {
            Ok(None)
        }
    }
    fn delete(&self, key: Key) -> Result<Option<Value>, ()> {
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
        let mut db = match self.db.write() {
            Ok(db) => db,
            Err(e) => panic!("write lock failed, {}", e.to_string()),
        };
        db.clear();
        Ok(true)
    }
}
