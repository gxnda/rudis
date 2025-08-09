use ahash::RandomState;
use bytes::Bytes;
use dashmap::mapref::entry::Entry;
use dashmap::{DashMap, DashSet};
use rayon::prelude::*;
use regex::bytes::Regex;
use serde::de::{self, Error, Visitor};
use serde::ser::{SerializeMap, SerializeSeq};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::VecDeque;
use std::fmt;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio_util::sync::CancellationToken;

#[derive(Default, Serialize, Deserialize)]
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
    #[serde(
        serialize_with = "serialize_expiry",
        deserialize_with = "deserialize_expiry"
    )]
    pub expiry: Option<Instant>,
}

#[derive(Clone)]
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

fn serialize_expiry<S>(expiry: &Option<Instant>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match *expiry {
        Some(exp) => {
            let now = Instant::now();
            if now >= exp {
                serializer.serialize_u128(0)
            } else {
                let remaining = exp - now;
                let total = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map_err(serde::ser::Error::custom)?
                    + remaining;
                serializer.serialize_u128(total.as_millis())
            }
        }
        None => serializer.serialize_u128(0),
    }
}

fn deserialize_expiry<'de, D>(deserializer: D) -> Result<Option<Instant>, D::Error>
where
    D: Deserializer<'de>,
{
    let millis = u128::deserialize(deserializer)?;
    if millis == 0 {
        return Ok(None);
    }

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(de::Error::custom)?;
    let expiry_duration = Duration::from_millis(millis as u64);

    if expiry_duration > now {
        let remaining = expiry_duration - now;
        Ok(Some(Instant::now() + remaining))
    } else {
        Ok(None)
    }
}

impl Serialize for RedisValue {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            RedisValue::Integer(i) => serializer.serialize_i64(*i),
            RedisValue::String(b) => serializer.serialize_bytes(b),
            RedisValue::List(list) => {
                let mut seq = serializer.serialize_seq(Some(list.len() + 1))?;
                seq.serialize_element(&1)?;
                for item in list {
                    seq.serialize_element(item)?;
                }
                seq.end()
            }
            RedisValue::Hash(hash) => {
                let mut map = serializer.serialize_map(Some(hash.len()))?;
                // not using par_iter bc serde isn't threadsafe
                for entry in hash.iter() {
                    map.serialize_entry(entry.key(), entry.value())?;
                }
                map.end()
            }
            RedisValue::Set(set) => {
                let mut seq = serializer.serialize_seq(Some(set.len() + 1))?;
                seq.serialize_element(&2)?;
                for item in set.iter() {
                    seq.serialize_element(&*item)?;
                }
                seq.end()
            }
        }
    }
}

impl<'de> Deserialize<'de> for RedisValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct RedisValueVisitor;
        impl<'de> Visitor<'de> for RedisValueVisitor {
            type Value = RedisValue;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("Redis Value, any of: Integer, String, Hash, List, Set.")
            }

            fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(RedisValue::Integer(v))
            }

            fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                if v <= i64::MAX as u64 {
                    Ok(RedisValue::Integer(v as i64))
                } else {
                    Err(Error::custom("u64 out of range for Integer"))
                }
            }

            fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
            where
                E: Error,
            {
                Ok(RedisValue::String(Bytes::from((*v).to_owned())))
            }

            fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Self::Value, E>
            where
                E: Error,
            {
                Ok(RedisValue::String(Bytes::from(v)))
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: Error,
            {
                Ok(RedisValue::String(Bytes::from(v.to_string())))
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: de::SeqAccess<'de>,
            {
                match seq.next_element::<i32>()? {
                    Some(2) => {
                        // Set
                        let set: DashSet<Bytes> = DashSet::new();
                        while let Some(elem) = seq.next_element::<Vec<u8>>()? {
                            set.insert(Bytes::from(elem));
                        }
                        Ok(RedisValue::Set(set))
                    }
                    _ => {
                        // List
                        let mut deque = VecDeque::new();
                        while let Some(elem) = seq.next_element::<Vec<u8>>()? {
                            deque.push_back(Bytes::from(elem));
                        }
                        Ok(RedisValue::List(deque))
                    }
                }
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: de::MapAccess<'de>,
            {
                let hash: DashMap<Bytes, Bytes> = DashMap::new();
                while let Some(entry) = map.next_entry()? {
                    hash.insert(entry.0, entry.1);
                }
                Ok(RedisValue::Hash(hash))
            }
        }

        deserializer.deserialize_any(RedisValueVisitor)
    }
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

impl DataEntry {
    pub fn clone(&self) -> Self {
        DataEntry {
            value: self.value.clone(),
            expiry: self.expiry.clone(),
        }
    }

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
        let now = Instant::now();

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

    pub fn remove_expired(&self) {
        self.data.retain(|_, entry| !entry.is_expired());
    }

    fn remove_expired_on(data: &DashMap<Bytes, DataEntry, RandomState>) {
        data.retain(|_, entry| !entry.is_expired());
    }

    pub fn remove_expired_par(&self) {
        // possibly faster for large dashmaps
        let keys_to_remove: Vec<_> = self
            .data
            .par_iter()
            .filter(|entry| entry.value().is_expired())
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
            let mut interval = tokio::time::interval(Duration::from_secs(1));
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
