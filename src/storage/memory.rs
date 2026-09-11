use ahash::RandomState;
use atoi::atoi;
use bytes::Bytes;
use coarsetime::{Clock, Duration};
use dashmap::mapref::entry::Entry;
use dashmap::{DashMap, DashSet};
use rayon::prelude::*;
use regex::bytes::Regex;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::VecDeque;
use std::sync::Arc;
use tokio_util::sync::CancellationToken;

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct StorageEngine {
    #[serde(
        serialize_with = "serialize_dashmap",
        deserialize_with = "deserialize_dashmap"
    )]
    data: Arc<DashMap<Bytes, DataEntry, ahash::RandomState>>,
    #[serde(skip)]
    cancel_token: CancellationToken,
}

#[derive(Serialize, Deserialize)]
pub struct DataEntry {
    pub value: RedisValue,
    pub expiry: Option<u64>,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum RedisValue {
    Integer(i64),
    String(Bytes),
    List(VecDeque<Bytes>),
    Hash(DashMap<Bytes, Bytes>),
    Set(DashSet<Bytes>),
}

fn serialize_dashmap<S>(
    data: &Arc<DashMap<Bytes, DataEntry, RandomState>>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let map: &DashMap<Bytes, DataEntry, RandomState> = data;
    map.serialize(serializer)
}

fn deserialize_dashmap<'de, D>(
    deserializer: D,
) -> Result<Arc<DashMap<Bytes, DataEntry, RandomState>>, D::Error>
where
    D: Deserializer<'de>,
{
    let map = DashMap::<Bytes, DataEntry, RandomState>::deserialize(deserializer)?;
    Ok(Arc::new(map))
}

impl RedisValue {
    pub fn as_integer(&self) -> Option<i64> {
        match self {
            RedisValue::Integer(i) => Some(*i),
            RedisValue::String(s) => atoi::<i64>(s),
            _ => None,
        }
    }
}

impl Clone for DataEntry {
    fn clone(&self) -> Self {
        DataEntry {
            value: self.value.clone(),
            expiry: self.expiry,
        }
    }
}

impl DataEntry {
    pub fn is_expired(&self) -> bool {
        self.is_older_than_now()
    }

    fn is_older_than(&self, instant: u64) -> bool {
        if let Some(exp) = self.expiry {
            exp < instant
        } else {
            false
        }
    }

    #[inline]
    fn is_older_than_now(&self) -> bool {
        self.is_older_than(Clock::recent_since_epoch().as_millis())
    }
}

pub enum IncrError {
    NotAnInteger,
    Overflow,
}

impl StorageEngine {
    pub fn with_capacity_and_shards(capacity: usize, shard_count: usize) -> Self {
        StorageEngine {
            data: Arc::new(DashMap::with_capacity_and_hasher_and_shard_amount(
                capacity,
                RandomState::new(),
                shard_count,
            )),
            cancel_token: CancellationToken::new(),
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        StorageEngine {
            data: Arc::new(DashMap::with_capacity_and_hasher(
                capacity,
                RandomState::new(),
            )),
            cancel_token: CancellationToken::new(),
        }
    }

    #[inline]
    pub fn get_at(&self, key: &Bytes, now: u64) -> Option<RedisValue> {
        match self.data.get(key) {
            Some(entry) if entry.is_older_than(now) => {
                drop(entry);
                self.data.remove(key);
                None
            }
            Some(entry) => Some(entry.value.clone()),
            None => None,
        }
    }

    pub fn get(&self, key: &Bytes) -> Option<RedisValue> {
        self.get_at(key, Clock::recent_since_epoch().as_millis())
    }

    pub fn set(&self, key: Bytes, value: RedisValue, expiry_instant: Option<u64>) {
        self.data.insert(
            key,
            DataEntry {
                value,
                expiry: expiry_instant,
            },
        );
    }

    #[inline]
    pub fn del(&self, key: &Bytes) -> bool {
        self.data.remove(key).is_some()
    }

    pub fn set_expire(&self, key: &Bytes, expiry_instant: Option<u64>) {
        if let Some(mut entry) = self.data.get_mut(key) {
            entry.expiry = expiry_instant;
        }
    }

    pub fn get_expire(&self, key: &Bytes) -> Result<Option<u64>, &'static str> {
        match self.data.get(key) {
            Some(entry) => Ok(entry.expiry), // return key, None if no expiry
            None => Err("key not found"),    // key doesn't exist
        }
    }

    #[inline]
    pub fn set_expire_in(&self, key: &Bytes, duration_ms: u64) {
        self.set_expire(
            key,
            Some(Clock::recent_since_epoch().as_millis() + duration_ms),
        );
    }

    #[inline]
    pub fn exists(&self, key: &Bytes) -> bool {
        self.data.contains_key(key)
    }

    #[inline]
    pub fn incr(&self, key: &Bytes) -> Result<i64, IncrError> {
        self.incr_by(key, 1)
    }

    pub fn incr_by(&self, key: &Bytes, incr: i64) -> Result<i64, IncrError> {
        match self.data.entry(key.clone()) {
            Entry::Occupied(e) => {
                let mut stored_val = e.into_ref();
                if stored_val.is_expired() {
                    stored_val.value = RedisValue::Integer(incr);
                    stored_val.expiry = None;
                    return Ok(incr);
                }
                match stored_val.value.as_integer() {
                    Some(i) => match &stored_val.value {
                        RedisValue::Integer(_) => {
                            let new_val = i.checked_add(incr).ok_or(IncrError::Overflow)?;
                            stored_val.value = RedisValue::Integer(new_val);
                            Ok(new_val)
                        }
                        RedisValue::String(_) => {
                            let new_val = i.checked_add(incr).ok_or(IncrError::Overflow)?;
                            let mut buffer = itoa::Buffer::new();
                            let printed = buffer.format(new_val);
                            stored_val.value =
                                RedisValue::String(Bytes::copy_from_slice(printed.as_bytes()));
                            Ok(new_val)
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
        let now = Clock::recent_since_epoch().as_millis();

        for entry in self.data.iter() {
            // check if its expired
            if entry.is_older_than(now) {
                continue;
            }

            if let RedisValue::String(str_bytes) = &entry.value {
                if re.is_match(str_bytes) {
                    // TODO: Doesn't this return the keys not values??
                    matches.push(entry.key().clone());
                }
            }
        }

        Ok(matches)
    }

    fn glob_to_regex(glob: &str) -> String {
        let mut regex = String::with_capacity(glob.len() * 2);
        regex.push('^'); // Match from the start

        for c in glob.chars() {
            match c {
                '*' => regex.push_str(".*"),
                '?' => regex.push('.'),
                '.' | '+' | '(' | ')' | '|' | '^' | '$' | '\\' => {
                    regex.push('\\');
                    regex.push(c);
                }
                '[' | ']' | '{' | '}' => regex.push(c), // Keep as-is (character classes)
                _ => regex.push(c),
            }
        }
        regex.push('$'); // Match to the end
        regex
    }

    pub fn get_matching_keys(&self, pattern: &str) -> Result<Vec<Bytes>, regex::Error> {
        // Returns all keys which match pattern
        let re = Regex::new(&StorageEngine::glob_to_regex(pattern))?;
        let now = Clock::recent_since_epoch().as_millis();

        Ok(self
            .data
            .iter()
            .filter(|entry| !entry.is_older_than(now))
            .filter(|entry| re.is_match(entry.key()))
            .map(|entry| entry.key().clone())
            .collect())
    }

    pub fn get_matching_keys_par(&self, pattern: &str) -> Result<Vec<Bytes>, regex::Error> {
        // parallel version of the above function using rayon
        let re = Regex::new(&StorageEngine::glob_to_regex(pattern))?;
        let now = Clock::recent_since_epoch().as_millis();
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

    pub fn remove_expired(&self) {
        self.remove_older_than(Clock::recent_since_epoch().as_millis())
    }

    pub fn remove_older_than(&self, inst: u64) {
        self.data.retain(|_, entry| !entry.is_older_than(inst));
    }

    fn remove_expired_on(data: &DashMap<Bytes, DataEntry, RandomState>) {
        let now = Clock::recent_since_epoch().as_millis();
        data.retain(|_, entry| !entry.is_older_than(now));
    }

    pub fn remove_expired_par(&self) {
        // possibly faster for large dashmaps
        let now = Clock::recent_since_epoch().as_millis();
        let keys_to_remove: Vec<_> = self
            .data
            .par_iter()
            .filter(|entry| entry.value().is_older_than(now))
            .map(|entry| entry.key().clone())
            .collect();

        for key in keys_to_remove {
            self.data.remove(&key);
        }
    }

    pub fn run_expiration_loop(&self) {
        let child_token = self.cancel_token.child_token();
        let data = Arc::clone(&self.data);

        // starts active expiration in bg
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(5).into());
            while !child_token.is_cancelled() {
                tokio::select! {
                    _ = child_token.cancelled() => break,
                    _ = interval.tick() => {
                        Self::remove_expired_on(&data);
                    }
                }
            }
        });
    }

    pub fn stop_expiration_loop(&self) {
        self.cancel_token.cancel();
    }
}
