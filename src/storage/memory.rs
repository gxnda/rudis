use ahash::RandomState;
use bytes::Bytes;
use dashmap::mapref::entry::Entry;
use dashmap::{DashMap, DashSet};
use rayon::prelude::*;
use regex::bytes::Regex;
use std::collections::VecDeque;
use std::time::{Duration, Instant};

#[derive(Default)]
pub struct StorageEngine {
    data: DashMap<Bytes, DataEntry, ahash::RandomState>,
}

pub struct DataEntry {
    pub value: RedisValue,
    pub expiry: Option<Instant>,
}

impl DataEntry {
    pub fn is_expired(&self) -> bool {
        self.is_older_than(Instant::now())
    }

    fn is_older_than(&self, instant: Instant) -> bool {
        if let Some(exp) = self.expiry {
            exp < instant
        } else {
            false
        }
    }
}

#[derive(Clone)]
pub enum RedisValue {
    Integer(i64),
    String(Bytes),
    List(VecDeque<Bytes>),
    Hash(DashMap<Bytes, Bytes>),
    Set(DashSet<Bytes>),
}

impl RedisValue {
    pub fn as_integer(&self) -> Option<i64> {
        match self {
            RedisValue::Integer(i) => Some(*i),
            RedisValue::String(s) => std::str::from_utf8(s).ok()?.parse().ok(),
            _ => None,
        }
    }
}

pub enum IncrError {
    NotAnInteger,
    Overflow,
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

    pub fn get(&self, key: &Bytes) -> Option<RedisValue> {
        if let Some(entry) = self.data.get(key) {
            // Passive expiration, deletes on get
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

    pub fn set(&self, key: Bytes, value: RedisValue, expiry: Option<Instant>) {
        self.data.insert(key, DataEntry { value, expiry });
    }

    pub fn del(&self, key: &Bytes) -> bool {
        self.data.remove(key).is_some()
    }

    pub fn set_expire(&self, key: &Bytes, expiry: Option<Instant>) {
        if let Some(mut entry) = self.data.get_mut(key) {
            entry.expiry = expiry;
        }
    }

    pub fn get_expire(&self, key: &Bytes) -> Result<Option<Instant>, &'static str> {
        match self.data.get(key) {
            Some(entry) => Ok(entry.expiry), // return key, None if no expiry
            None => Err("key not found"),    // key doesn't exist
        }
    }

    pub fn set_expire_in(&self, key: &Bytes, duration: Duration) {
        self.set_expire(key, Some(Instant::now() + duration));
    }

    pub fn exists(&self, key: &Bytes) -> bool {
        self.data.contains_key(key)
    }

    pub fn incr(&self, key: &Bytes) -> Result<i64, IncrError> {
        self.incr_by(key, 1)
    }

    pub fn incr_by(&self, key: &Bytes, incr: i64) -> Result<i64, IncrError> {
        match self.data.entry(key.clone()) {
            Entry::Occupied(e) => {
                let mut stored_val = e.into_ref();
                if stored_val.is_expired() {
                    todo!();
                }
                match stored_val.value.as_integer() {
                    Some(i) => match stored_val.value {
                        RedisValue::Integer(_) => {
                            stored_val.value = RedisValue::Integer(i + incr);
                            Ok(i + incr)
                        }
                        RedisValue::String(_) => {
                            stored_val.value =
                                RedisValue::String(Bytes::from((i + incr).to_string()));
                            Ok(i + incr)
                        }
                        _ => Err(IncrError::NotAnInteger),
                    },
                    None => Err(IncrError::NotAnInteger),
                }
            }
            Entry::Vacant(e) => {
                e.insert_entry(DataEntry {
                    value: RedisValue::Integer(incr),
                    expiry: None,
                });
                Ok(incr)
            }
        }
    }

    pub fn decr(&self, key: &Bytes) -> Result<i64, IncrError> {
        self.incr_by(key, -1)
    }

    pub fn alter(&self, key: &Bytes, f: impl FnOnce(&Bytes, DataEntry) -> DataEntry) {
        self.data.alter(key, f)
    }

    pub fn clear(&self) {
        self.data.clear()
    }

    pub fn get_matching_values(&self, pattern: &str) -> Result<Vec<Bytes>, regex::Error> {
        // Returns all keys with matching values
        let re = Regex::new(pattern)?;
        let mut matches: Vec<Bytes> = Vec::new();
        let now = Instant::now();

        for entry in self.data.iter() {
            // check if its expired
            if let Some(expiry) = entry.expiry {
                if expiry < now {
                    continue;
                }
            }

            if let RedisValue::String(str_bytes) = &entry.value {
                if re.is_match(str_bytes) {
                    matches.push(entry.key().clone());
                }
            }
        }
        Ok(matches)
    }

    pub fn get_matching_keys(&self, pattern: &str) -> Result<Vec<Bytes>, regex::Error> {
        // Returns all keys which match pattern
        let re = Regex::new(pattern)?;
        let mut matches: Vec<Bytes> = Vec::new();
        let now = Instant::now();

        for entry in self.data.iter() {
            // check if its expired
            if entry.is_older_than(now) {
                continue;
            }

            if re.is_match(entry.key()) {
                matches.push(entry.key().clone());
            }
        }
        Ok(matches)
    }

    pub fn get_matching_keys_par(&self, pattern: &str) -> Result<Vec<Bytes>, regex::Error> {
        // parallel version of the above function using rayon
        let re = Regex::new(pattern)?;
        let now = Instant::now();
        Ok(self
            .data
            .par_iter()
            .filter_map(|entry| {
                if entry.is_older_than(now) {
                    None
                } else {
                    re.is_match(entry.key()).then(|| entry.key().clone())
                }
            })
            .collect())
    }
}
