pub mod btreemap_engine;
pub mod engine_trait;

// KV database's data type
pub type Key = String;
pub type Value = String;

use self::btreemap_engine::BtreeMapEngine;
use self::engine_trait::Engine;

/// Additional layer of packaging for easy expansion
/// For example, store other information about databases, such as names
///
/// Generic E needs to implement Engine trait
#[derive(Clone)]
pub struct Database<E: Engine> {
    pub engine: E,
    name: String,
}

impl Database<BtreeMapEngine> {
    /// Constructs a new, empty `Database`.
    ///
    /// The default name is `BtreeMapEngine_Storage`
    pub fn new() -> Self {
        let db = Database {
            engine: BtreeMapEngine::new(),
            name: "BtreeMapEngine_Storage".to_string(),
        };
        db
    }
    /// Set the database name
    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }
    /// Get the database name
    pub fn get_name(&self) -> &String {
        &self.name
    }
}