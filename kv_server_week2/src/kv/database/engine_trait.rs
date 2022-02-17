use super::{Key, Value};
use std::collections::HashMap;

/// Define a trait of Engine
///
/// Functions include get,set,insert,update,delete,scan,scan_all and clear.
pub trait Engine {
    fn get(&self, key: Key) -> Result<Option<Value>, ()>;
    fn set(&self, key: Key, value: Value) -> Result<Option<Value>, ()>;
    fn insert(&self, key: Key, value: Value) -> Result<bool, ()>;
    fn update(&self, key: Key, value: Value) -> Result<Option<Value>, ()>;
    fn delete(&self, key: Key) -> Result<Option<Value>, ()>;
    fn scan(&self, key_start: Key, key_end: Key) -> Result<Option<HashMap<Key, Value>>, ()>;
    fn scan_all(&self) -> Result<Option<HashMap<Key, Value>>, ()>;
    fn clear(&self) -> Result<bool, ()>;
}
