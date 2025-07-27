use ahash::RandomState;
use bytes::Bytes;
use dashmap::DashMap;
use std::time::{Duration, Instant};

pub struct StorageEngine {
    data: DashMap<Bytes, DataEntry, RandomState>,
}

struct DataEntry {
    value: Bytes,
    expiry: Option<Instant>,
}

pub enum IncrError {
    NotAnInteger,
    Overflow
}

impl StorageEngine {
    pub fn with_capacity_and_shards(capacity: usize, shard_count: usize) -> Self {
        StorageEngine {
            data: DashMap::with_capacity_and_hasher_and_shard_amount(
                capacity,
                RandomState::new(),
                shard_count,
            ),
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        StorageEngine {
            data: DashMap::with_capacity_and_hasher(capacity, RandomState::new()),
        }
    }

    pub fn get(&self, key: &Bytes) -> Option<Bytes> {
        if let Some(entry) = self.data.get(key) {
            if let Some(expiry) = entry.expiry {
                if expiry < Instant::now() {
                    drop(entry);
                    self.data.remove(key);
                    return None;
                }
            }
            return Some(entry.value.clone());
        }
        None
    }

    pub fn set(&self, key: Bytes, value: Bytes, expiry: Option<Instant>) {
        self.data.insert(key, DataEntry { value, expiry });
    }

    pub fn del(&self, key: &Bytes) {
        self.data.remove(key);
    }

    pub fn set_expire(&self, key: &Bytes, expiry: Instant) {
        if let Some(mut entry) = self.data.get_mut(key) {
            entry.expiry = Some(expiry);
        }
    }

    pub fn set_expire_in(&self, key: &Bytes, duration: Duration) {
        self.set_expire(key, Instant::now() + duration);
    }

    pub fn exists(&self, key: &Bytes) -> bool {
        self.data.contains_key(key)
    }

    pub fn incr(&self, key: &Bytes) -> Result<i64, IncrError> {
        self.incr_by(key, 1)
    }

    pub fn incr_by(&self, key: &Bytes, incr: i64) -> Result<i64, IncrError> {
        let mut entry = self.data.entry(key.clone()).or_insert_with(|| DataEntry {
            value: Bytes::from("0"),
            expiry: None,
        });

        if let Some(expiry) = entry.expiry {
            if expiry < Instant::now() {
                entry.value = Bytes::from("0");
                entry.expiry = None;
            }
        }

        let current_str = std::str::from_utf8(&entry.value).map_err(|_| IncrError::NotAnInteger)?;
        let current_value: i64 = current_str.parse().map_err(|_| IncrError::NotAnInteger)?;

        let new_value = current_value.checked_add(incr).ok_or(IncrError::Overflow)?;
        entry.value = Bytes::from(new_value.to_string());
        Ok(new_value)
    }

    pub fn decr(&self, key: &Bytes) -> Result<i64, IncrError> {
        self.incr_by(key, -1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::thread;

    #[test]
    fn test_set_get() {
        let engine = StorageEngine::with_capacity(10);
        let key = Bytes::from("key");
        let value = Bytes::from("value");

        // make sure an empty storage returns none
        assert!(engine.get(&key).is_none());

        engine.set(key.clone(), value.clone(), None);
        assert_eq!(engine.get(&key), Some(value));

        // overwrite
        let new_value = Bytes::from("new_value");
        engine.set(key.clone(), new_value.clone(), None);
        assert_eq!(engine.get(&key), Some(new_value));
    }

    #[test]
    fn test_del() {
        let engine = StorageEngine::with_capacity(10);
        let key = Bytes::from("key");
        let value = Bytes::from("value");

        engine.set(key.clone(), value, None);
        engine.del(&key);
        assert!(engine.get(&key).is_none());
    }

    #[test]
    fn test_expiry_immediate() {
        let engine = StorageEngine::with_capacity(10);
        let key = Bytes::from("key");
        let value = Bytes::from("value");

        // Set with immediate expiry (past)
        engine.set(
            key.clone(),
            value,
            Some(Instant::now() - Duration::from_secs(1)),
        );
        assert!(engine.get(&key).is_none());
    }

    #[test]
    fn test_expiry_ttl() {
        let engine = StorageEngine::with_capacity(10);
        let key = Bytes::from("key");
        let value = Bytes::from("value");

        // Set with short TTL
        engine.set(
            key.clone(),
            value.clone(),
            Some(Instant::now() + Duration::from_millis(10)),
        );
        assert_eq!(engine.get(&key), Some(value.clone()));

        // Wait for expiry and verify cleanup
        thread::sleep(Duration::from_millis(20));
        assert!(engine.get(&key).is_none());
    }

    #[test]
    fn test_expiry_override() {
        let engine = StorageEngine::with_capacity(10);
        let key = Bytes::from("key");
        let value = Bytes::from("value");

        // Set initial expiry (short)
        engine.set(
            key.clone(),
            value.clone(),
            Some(Instant::now() + Duration::from_millis(10)),
        );

        // Extend expiry
        engine.set_expire_in(&key, Duration::from_millis(30));
        thread::sleep(Duration::from_millis(20));

        // Should still exist
        assert_eq!(engine.get(&key), Some(value));
    }

    #[test]
    fn test_set_expire_non_existent() {
        let engine = StorageEngine::with_capacity(10);
        let key = Bytes::from("key");

        // No-op on non-existent key
        engine.set_expire(&key, Instant::now() + Duration::from_secs(10));
        engine.set_expire_in(&key, Duration::from_secs(10));
    }

    #[test]
    fn test_concurrent_access() {
        let engine = Arc::new(StorageEngine::with_capacity_and_shards(100, 32));
        let mut handles = Vec::new();

        // Spawn 10 threads writing unique keys
        for i in 0..10 {
            let engine_clone = engine.clone();
            handles.push(thread::spawn(move || {
                let key = Bytes::from(format!("key_{}", i));
                let value = Bytes::from(format!("value_{}", i));
                engine_clone.set(key, value, None);
            }));
        }

        // Spawn 10 threads reading keys (may not exist yet)
        for i in 0..10 {
            let engine_clone = engine.clone();
            handles.push(thread::spawn(move || {
                let key = Bytes::from(format!("key_{}", i));
                for _ in 0..100 {
                    engine_clone.get(&key);
                }
            }));
        }

        // Wait for all threads
        for handle in handles {
            handle.join().unwrap();
        }

        // Verify all writes are visible
        for i in 0..10 {
            let key = Bytes::from(format!("key_{}", i));
            assert_eq!(engine.get(&key).unwrap(), format!("value_{}", i));
        }
    }

    #[test]
    fn test_expire_in_zero_duration() {
        let engine = StorageEngine::with_capacity(10);
        let key = Bytes::from("key");
        let value = Bytes::from("value");

        engine.set(key.clone(), value, None);
        engine.set_expire_in(&key, Duration::from_secs(0)); // Expire immediately

        // Verify key is removed on next access
        assert!(engine.get(&key).is_none());
    }
}
