use crate::storage::memory::{DataEntry, RedisValue, StorageEngine};
use crate::{resp::RespValue, storage::memory::IncrError};
use bytes::Bytes;
use std::collections::VecDeque;
use std::time::{Duration, Instant};
use thiserror::Error;

#[derive(Debug)]
pub enum Command {
    Invalid {
        message: Option<Bytes>,
    },
    Get {
        key: Bytes,
    },
    Set {
        key: Bytes,
        value: Bytes,
        ttl: Option<Instant>,
    },
    Del {
        keys: Vec<Bytes>,
    },
    Exists {
        keys: Vec<Bytes>,
    },
    Incr {
        key: Bytes,
    },
    Decr {
        key: Bytes,
    },
    Ping {
        message: Option<Bytes>,
    },
    Echo {
        message: Bytes,
    },
    FlushAll {},
    Keys {
        pattern: String,
    }, // finds keys matching given pattern
    TTL {
        key: Bytes,
    },
    Expire {
        key: Bytes,
        ttl: Duration,
    },
    Persist {
        key: Bytes,
    },
    LPush {
        key: Bytes,
        elements: Vec<Bytes>,
    },
    RPush {
        key: Bytes,
        elements: Vec<Bytes>,
    },
    LPop {
        key: Bytes,
        count: Option<u64>,
    },
    RPop {
        key: Bytes,
        count: Option<u64>,
    },
    LLen {
        key: Bytes,
    },
}

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("Invalid command: {0}")]
    InvalidCommand(String),

    #[error("Invalid arg count, expected: {expected}, got: {got}")]
    InvalidArgCount { expected: usize, got: usize },
    #[error("Invalid argument: {0}")]
    InvalidArgument(String),
    #[error("Invalid duration: {0}")]
    InvalidDuration(String),
}

impl Command {
    pub fn from_resp(resp: RespValue) -> Result<Self, ParseError> {
        // Takes in a RespValue array of bulk strings, since that is what is given in the request
        // e.g. [b"SET", 234, b"bar"]
        let mut args = match resp {
            RespValue::Array(Some(vec)) => vec,
            RespValue::Array(None) => {
                return Err(ParseError::InvalidCommand("Null array".to_string()))
            }
            _ => return Err(ParseError::InvalidCommand("Expected array".to_string())),
        };
        if args.is_empty() {
            return Err(ParseError::InvalidCommand(
                "Empty command array".to_string(),
            ));
        }
        let cmd_value = args.remove(0);
        let cmd_bytes = match cmd_value {
            RespValue::BulkString(Some(s)) => s,
            _ => {
                return Err(ParseError::InvalidCommand(
                    "Command must be bulk string".to_string(),
                ))
            }
        };
        let cmd = String::from_utf8_lossy(&cmd_bytes).to_uppercase();

        let mut arg_bytes = Vec::with_capacity(args.len());
        for arg in args {
            match arg {
                RespValue::BulkString(Some(s)) => arg_bytes.push(Bytes::from(s)),
                RespValue::BulkString(None) => {
                    return Err(ParseError::InvalidArgument(
                        "Null bulk string not allowed".to_string(),
                    ))
                }
                _ => {
                    return Err(ParseError::InvalidArgument(
                        "Expected bulk string".to_string(),
                    ))
                }
            }
        }

        match cmd.as_str() {
            "GET" => {
                if arg_bytes.len() != 1 {
                    return Err(ParseError::InvalidArgCount {
                        expected: 1,
                        got: arg_bytes.len(),
                    });
                }
                Ok(Command::Get {
                    key: arg_bytes[0].clone(),
                })
            }
            "SET" => {
                if arg_bytes.len() < 2 {
                    return Err(ParseError::InvalidArgCount {
                        expected: 2,
                        got: arg_bytes.len(),
                    });
                }
                let key = arg_bytes[0].clone();
                let value = arg_bytes[1].clone();
                let mut ttl = None;

                // Process optional TTL arguments
                let mut i = 2;
                while i < arg_bytes.len() {
                    let opt_str = String::from_utf8_lossy(&arg_bytes[i]).to_uppercase();
                    match opt_str.as_str() {
                        "EX" | "PX" => {
                            if i + 1 >= arg_bytes.len() {
                                return Err(ParseError::InvalidArgument(format!(
                                    "Missing value for {} option",
                                    opt_str
                                )));
                            }
                            let num_str = std::str::from_utf8(&arg_bytes[i + 1]).map_err(|_| {
                                ParseError::InvalidArgument("Invalid UTF-8".to_string())
                            })?;
                            let duration_val = num_str.parse::<u64>().map_err(|_| {
                                ParseError::InvalidDuration("Invalid duration value".to_string())
                            })?;

                            ttl = Some(match opt_str.as_str() {
                                "EX" => Instant::now() + Duration::from_secs(duration_val),
                                "PX" => Instant::now() + Duration::from_millis(duration_val),
                                _ => unreachable!(),
                            });
                            i += 2; // Skip option and its value
                        }
                        _ => {
                            return Err(ParseError::InvalidArgument(format!(
                                "Unsupported option: {}",
                                opt_str
                            )))
                        }
                    }
                }

                Ok(Command::Set { key, value, ttl })
            }

            "DEL" => {
                if arg_bytes.is_empty() {
                    return Err(ParseError::InvalidArgCount {
                        expected: 1,
                        got: 0,
                    });
                }
                Ok(Command::Del { keys: arg_bytes })
            }

            "EXISTS" => {
                if arg_bytes.is_empty() {
                    return Err(ParseError::InvalidArgCount {
                        expected: 1,
                        got: 0,
                    });
                }
                Ok(Command::Exists { keys: arg_bytes })
            }

            "INCR" => {
                if arg_bytes.len() != 1 {
                    return Err(ParseError::InvalidArgCount {
                        expected: 1,
                        got: arg_bytes.len(),
                    });
                }
                Ok(Command::Incr {
                    key: arg_bytes[0].clone(),
                })
            }

            "DECR" => {
                if arg_bytes.len() != 1 {
                    return Err(ParseError::InvalidArgCount {
                        expected: 1,
                        got: arg_bytes.len(),
                    });
                }
                Ok(Command::Decr {
                    key: arg_bytes[0].clone(),
                })
            }

            "PING" => {
                let message = if arg_bytes.is_empty() {
                    None
                } else {
                    Some(arg_bytes[0].clone())
                };
                Ok(Command::Ping { message })
            }

            "ECHO" => {
                if arg_bytes.len() != 1 {
                    return Err(ParseError::InvalidArgCount {
                        expected: 1,
                        got: arg_bytes.len(),
                    });
                }
                Ok(Command::Echo {
                    message: arg_bytes[0].clone(),
                })
            }

            "FLUSHALL" => {
                if !arg_bytes.is_empty() {
                    return Err(ParseError::InvalidArgCount {
                        expected: 0,
                        got: arg_bytes.len(),
                    });
                }
                Ok(Command::FlushAll {})
            }

            "KEYS" => {
                if arg_bytes.len() != 1 {
                    return Err(ParseError::InvalidArgCount {
                        expected: 1,
                        got: arg_bytes.len(),
                    });
                }
                let pattern = String::from_utf8_lossy(&arg_bytes[0]).to_string();
                Ok(Command::Keys { pattern })
            }

            "TTL" => {
                if arg_bytes.len() != 1 {
                    return Err(ParseError::InvalidArgCount {
                        expected: 1,
                        got: arg_bytes.len(),
                    });
                }
                Ok(Command::TTL {
                    key: arg_bytes[0].clone(),
                })
            }

            "EXPIRE" => {
                if arg_bytes.len() != 2 {
                    return Err(ParseError::InvalidArgCount {
                        expected: 2,
                        got: arg_bytes.len(),
                    });
                }
                let key = arg_bytes[0].clone();
                let seconds: u64 =
                    String::from_utf8_lossy(&arg_bytes[1])
                        .parse()
                        .map_err(|_| {
                            ParseError::InvalidDuration("Invalid expire seconds".to_string())
                        })?;
                Ok(Command::Expire {
                    key,
                    ttl: Duration::from_secs(seconds),
                })
            }

            "PERSIST" => {
                if arg_bytes.len() != 1 {
                    return Err(ParseError::InvalidArgCount {
                        expected: 1,
                        got: arg_bytes.len(),
                    });
                }
                Ok(Command::Persist {
                    key: arg_bytes[0].clone(),
                })
            }

            "LPUSH" => {
                if arg_bytes.len() < 2 {
                    return Err(ParseError::InvalidArgCount {
                        expected: 2,
                        got: arg_bytes.len(),
                    });
                }
                let key = arg_bytes[0].clone();
                let elements = arg_bytes[1..].to_vec();
                Ok(Command::LPush { key, elements })
            }

            "RPUSH" => {
                if arg_bytes.len() < 2 {
                    return Err(ParseError::InvalidArgCount {
                        expected: 2,
                        got: arg_bytes.len(),
                    });
                }
                let key = arg_bytes[0].clone();
                let elements = arg_bytes[1..].to_vec();
                Ok(Command::RPush { key, elements })
            }

            "LPOP" => match arg_bytes.len() {
                1 => Ok(Command::LPop {
                    key: arg_bytes[0].clone(),
                    count: None,
                }),
                2 => {
                    let count = std::str::from_utf8(&arg_bytes[1])
                        .map_err(|_| ParseError::InvalidArgument("Invalid count".to_string()))?
                        .parse::<u64>()
                        .map_err(|_| ParseError::InvalidArgument("Invalid count".to_string()))?;
                    Ok(Command::LPop {
                        key: arg_bytes[0].clone(),
                        count: Some(count),
                    })
                }
                _ => Err(ParseError::InvalidArgCount {
                    expected: 1,
                    got: arg_bytes.len(),
                }),
            },

            "RPOP" => match arg_bytes.len() {
                1 => Ok(Command::RPop {
                    key: arg_bytes[0].clone(),
                    count: None,
                }),
                2 => {
                    let count = std::str::from_utf8(&arg_bytes[1])
                        .map_err(|_| ParseError::InvalidArgument("Invalid count".to_string()))?
                        .parse::<u64>()
                        .map_err(|_| ParseError::InvalidArgument("Invalid count".to_string()))?;
                    Ok(Command::RPop {
                        key: arg_bytes[0].clone(),
                        count: Some(count),
                    })
                }
                _ => Err(ParseError::InvalidArgCount {
                    expected: 1,
                    got: arg_bytes.len(),
                }),
            },

            "LLEN" => {
                if arg_bytes.len() != 1 {
                    return Err(ParseError::InvalidArgCount {
                        expected: 1,
                        got: arg_bytes.len(),
                    });
                }
                Ok(Command::LLen {
                    key: arg_bytes[0].clone(),
                })
            }

            _ => Err(ParseError::InvalidCommand(cmd)),
        }
    }

    pub fn execute(&self, storage: &StorageEngine) -> RespValue {
        match self {
            Command::Invalid { message: _ } => RespValue::Error("Invalid message: ".to_string()),
            Command::Get { key } => {
                // Bulk string reply if exists
                // Nil reply if not
                match storage.get(key) {
                    Some(RedisValue::String(s)) => RespValue::BulkString(Some(s)),
                    None => RespValue::BulkString(None),
                    _ => RespValue::Error(
                        "WRONGTYPE Operation against a key holding the wrong kind of value"
                            .to_string(),
                    ),
                }
            }
            Command::Set { key, value, ttl } => {
                // ttl can be none, ONLY WORKS FOR STRINGS
                storage.set(key.clone(), RedisValue::String(value.clone()), *ttl);
                RespValue::SimpleString("OK".to_string())
            }
            Command::Del { keys } => {
                let mut count = 0;
                for key in keys {
                    if storage.del(key) {
                        count += 1;
                    }
                }
                RespValue::Integer(count)
            }
            Command::Exists { keys } => {
                let mut count: i64 = 0;
                for key in keys {
                    if storage.exists(key) {
                        count += 1;
                    }
                }
                RespValue::Integer(count)
            }
            Command::Incr { key } => {
                // Sets to 0 if doesn't exist
                // Otherwise increments by 1
                // Only on i64
                match storage.incr(key) {
                    Ok(new_value) => RespValue::Integer(new_value),
                    Err(IncrError::NotAnInteger) => RespValue::Error(
                        "Error attempting to increment value, was not an integer".to_string(),
                    ),
                    Err(IncrError::Overflow) => RespValue::Error(
                        "Error attempting to increment value, overflow".to_string(),
                    ),
                }
            }

            Command::Decr { key } => match storage.decr(key) {
                Ok(new_value) => RespValue::Integer(new_value),
                Err(IncrError::NotAnInteger) => RespValue::Error(
                    "Error attempting to increment value, was not an integer".to_string(),
                ),
                Err(IncrError::Overflow) => {
                    RespValue::Error("Error attempting to increment value, underflow".to_string())
                }
            },

            Command::Ping { message } => {
                if message.is_some() {
                    RespValue::BulkString(message.clone())
                } else {
                    RespValue::SimpleString("PONG".to_string())
                }
            }

            Command::Echo { message } => RespValue::BulkString(Some(message.clone())),

            Command::FlushAll {} => {
                storage.clear();
                RespValue::SimpleString("OK".to_string())
            }

            Command::Keys { pattern } => match storage.get_matching_keys(pattern) {
                Ok(keys) => RespValue::Array(Some(
                    keys.iter()
                        .map(|key| RespValue::BulkString(Some(key.clone())))
                        .collect(),
                )),
                Err(e) => RespValue::Error(e.to_string()),
            },

            Command::TTL { key } => match storage.get_expire(key) {
                Ok(Some(exp)) => {
                    RespValue::Integer(exp.duration_since(Instant::now()).as_secs() as i64)
                }
                Ok(None) => RespValue::Integer(-1),
                Err(_) => RespValue::Integer(-2),
            },

            Command::Expire { key, ttl } => {
                if storage.exists(key) {
                    // sets expire for key
                    storage.set_expire_in(key, *ttl);
                    RespValue::Integer(1)
                } else {
                    RespValue::Integer(0)
                }
            }

            Command::Persist { key } => {
                if !storage.exists(key) {
                    return RespValue::Integer(0);
                }
                if storage.get_expire(key).is_err() {
                    return RespValue::Integer(0);
                }
                storage.set_expire(key, None);
                RespValue::Integer(1)
            }

            Command::LPush { key, elements } => {
                if storage.exists(key) {
                    let mut len = 0;
                    let mut type_error = false;
                    storage.alter(key, |_, val| match val.value {
                        RedisValue::List(mut list) => {
                            len = list.len() + elements.len();
                            let mut new_elements = VecDeque::with_capacity(elements.len());
                            new_elements.extend(elements.iter().rev().cloned());
                            new_elements.append(&mut list);

                            DataEntry {
                                value: RedisValue::List(new_elements),
                                expiry: val.expiry,
                            }
                        }

                        _ => {
                            type_error = true;
                            val
                        }
                    });
                    if type_error {
                        RespValue::Error(
                            "WRONGTYPE Operation against a key holding the wrong kind of value"
                                .to_string(),
                        )
                    } else {
                        RespValue::Integer(len as i64)
                    }
                } else {
                    let list = elements.iter().rev().cloned().collect();
                    storage.set(key.clone(), RedisValue::List(list), None);
                    RespValue::Integer(elements.len() as i64)
                }
            }

            Command::RPush { key, elements } => {
                if storage.exists(&key) {
                    let mut new_len = 0;
                    let mut type_error = false;

                    storage.alter(&key, |_, val| match val.value {
                        RedisValue::List(mut list) => {
                            new_len = list.len() + elements.len();
                            list.extend(elements.iter().cloned());

                            DataEntry {
                                value: RedisValue::List(list),
                                expiry: val.expiry,
                            }
                        }
                        _ => {
                            type_error = true;
                            val
                        }
                    });

                    if type_error {
                        RespValue::Error(
                            "WRONGTYPE Operation against a key holding the wrong kind of value"
                                .to_string(),
                        )
                    } else {
                        RespValue::Integer(new_len as i64)
                    }
                } else {
                    // Create new list with elements in insertion order
                    let list = elements.iter().cloned().collect();
                    storage.set(key.clone(), RedisValue::List(list), None);
                    RespValue::Integer(elements.len() as i64)
                }
            }

            Command::LPop { key, count } => match storage.get(key) {
                Some(RedisValue::List(_)) => match count {
                    Some(count) => {
                        let mut popped = vec![];
                        storage.alter(key, |_, mut val| {
                            if let RedisValue::List(list) = &mut val.value {
                                for _ in 0..(*count).min(list.len() as u64) {
                                    popped.push(RespValue::BulkString(list.pop_front()));
                                }
                            }
                            val
                        });
                        RespValue::Array(Some(popped))
                    }
                    None => {
                        let mut result = RespValue::BulkString(None);
                        storage.alter(key, |_, mut val| {
                            if let RedisValue::List(list) = &mut val.value {
                                result = RespValue::BulkString(list.pop_front());
                            }
                            val
                        });
                        result
                    }
                },
                _ => RespValue::BulkString(None),
            },

            Command::RPop { key, count } => match storage.get(key) {
                Some(RedisValue::List(_)) => match count {
                    Some(count) => {
                        let mut popped = vec![];
                        storage.alter(key, |_, mut val| {
                            if let RedisValue::List(list) = &mut val.value {
                                for _ in 0..(*count).min(list.len() as u64) {
                                    popped.push(RespValue::BulkString(list.pop_back()));
                                }
                            }
                            val
                        });
                        RespValue::Array(Some(popped))
                    }
                    None => {
                        let mut result = RespValue::BulkString(None);
                        storage.alter(key, |_, mut val| {
                            if let RedisValue::List(list) = &mut val.value {
                                result = RespValue::BulkString(list.pop_back());
                            }
                            val
                        });
                        result
                    }
                },
                _ => RespValue::BulkString(None),
            },

            Command::LLen { key } => match storage.get(key) {
                Some(RedisValue::List(arr)) => {
                    println!("{:?}", arr);
                    RespValue::Integer(arr.len() as i64)
                }
                None => RespValue::Integer(0),
                _ => RespValue::Error(
                    "WRONGTYPE Operation against a key holding the wrong kind of value".to_string(),
                ),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::memory::StorageEngine;
    use bytes::Bytes;
    use std::time::Instant;

    // Helper to create RESP command array
    fn resp_array(args: Vec<&[u8]>) -> RespValue {
        RespValue::Array(Some(
            args.into_iter()
                .map(|a| RespValue::BulkString(Some(a.to_vec().into())))
                .collect(),
        ))
    }

    #[test]
    fn test_invalid_commands() {
        // Empty command
        let resp = RespValue::Array(None);
        assert!(matches!(
            Command::from_resp(resp),
            Err(ParseError::InvalidCommand(_))
        ));

        // Non-array command
        let resp = RespValue::SimpleString("SET foo bar".to_string());
        assert!(matches!(
            Command::from_resp(resp),
            Err(ParseError::InvalidCommand(_))
        ));

        // Unknown command
        let resp = resp_array(vec![b"FOOBAR", b"key"]);
        assert!(matches!(
            Command::from_resp(resp),
            Err(ParseError::InvalidCommand(_))
        ));
    }

    #[test]
    fn test_get_set() {
        let storage = StorageEngine::with_capacity(100);
        let key = b"test_key";
        let value = b"test_value";

        // Test SET
        let set_cmd = resp_array(vec![b"SET", key, value]);
        let cmd = Command::from_resp(set_cmd).unwrap();
        assert!(matches!(cmd, Command::Set { .. }));
        let response = cmd.execute(&storage);
        assert_eq!(response, RespValue::SimpleString("OK".to_string()));

        // Test GET
        let get_cmd = resp_array(vec![b"GET", key]);
        let cmd = Command::from_resp(get_cmd).unwrap();
        let response = cmd.execute(&storage);
        assert_eq!(
            response,
            RespValue::BulkString(Some(Bytes::copy_from_slice(value)))
        );

        // Test GET non-existent
        let get_cmd = resp_array(vec![b"GET", b"missing"]);
        let cmd = Command::from_resp(get_cmd).unwrap();
        let response = cmd.execute(&storage);
        assert_eq!(response, RespValue::BulkString(None));
    }

    #[test]
    fn test_set_with_ttl() {
        let key = b"expiring_key";
        let value = b"value";

        // SET with EX
        let set_cmd = resp_array(vec![b"SET", key, value, b"EX", b"10"]);
        let cmd = Command::from_resp(set_cmd).unwrap();
        if let Command::Set { ttl, .. } = cmd {
            assert!(ttl.is_some());
            assert!(ttl.unwrap() > Instant::now());
        } else {
            panic!("Not a SET command");
        }

        // Invalid TTL value
        let set_cmd = resp_array(vec![b"SET", key, value, b"EX", b"not_a_number"]);
        assert!(matches!(
            Command::from_resp(set_cmd),
            Err(ParseError::InvalidDuration(_))
        ));
    }

    #[test]
    fn test_del_exists() {
        let storage = StorageEngine::with_capacity(100);
        let keys = vec![b"k1", b"k2", b"k3"];

        // Set keys
        for key in &keys {
            storage.set(
                Bytes::from_static(*key),
                RedisValue::String(Bytes::from_static(b"v")),
                None,
            );
        }

        // DEL multiple keys
        let del_cmd = resp_array(vec![b"DEL", keys[0], keys[1], b"missing"]);
        let cmd = Command::from_resp(del_cmd).unwrap();
        let response = cmd.execute(&storage);
        assert_eq!(response, RespValue::Integer(2)); // 2 keys deleted

        // EXISTS check
        let exists_cmd = resp_array(vec![b"EXISTS", keys[0], keys[1], keys[2]]);
        let cmd = Command::from_resp(exists_cmd).unwrap();
        let response = cmd.execute(&storage);
        assert_eq!(response, RespValue::Integer(1)); // Only keys[2] exists
    }

    #[test]
    fn test_incr_decr() {
        let storage = StorageEngine::with_capacity(100);
        let key = b"counter";

        // INCR new key
        let incr_cmd = resp_array(vec![b"INCR", key]);
        let cmd = Command::from_resp(incr_cmd).unwrap();
        let response = cmd.execute(&storage);
        assert_eq!(response, RespValue::Integer(1));

        // DECR existing key
        let decr_cmd = resp_array(vec![b"DECR", key]);
        let cmd = Command::from_resp(decr_cmd).unwrap();
        let response = cmd.execute(&storage);
        assert_eq!(response, RespValue::Integer(0));

        // Test type error
        storage.set(
            Bytes::from_static(key),
            RedisValue::String(Bytes::from_static(b"not_int")),
            None,
        );
        let incr_cmd = resp_array(vec![b"INCR", key]);
        let cmd = Command::from_resp(incr_cmd).unwrap();
        let response = cmd.execute(&storage);
        assert!(matches!(response, RespValue::Error(_)));
    }

    #[test]
    fn test_ping_echo() {
        let storage = StorageEngine::with_capacity(100);

        // PING without message
        let ping_cmd = resp_array(vec![b"PING"]);
        let cmd = Command::from_resp(ping_cmd).unwrap();
        let response = cmd.execute(&storage);
        assert_eq!(response, RespValue::SimpleString("PONG".to_string()));

        // PING with message
        let ping_cmd = resp_array(vec![b"PING", b"Hello"]);
        let cmd = Command::from_resp(ping_cmd).unwrap();
        let response = cmd.execute(&storage);
        assert_eq!(
            response,
            RespValue::BulkString(Some(Bytes::from_static(b"Hello")))
        );

        // ECHO
        let echo_cmd = resp_array(vec![b"ECHO", b"Hello"]);
        let cmd = Command::from_resp(echo_cmd).unwrap();
        let response = cmd.execute(&storage);
        assert_eq!(
            response,
            RespValue::BulkString(Some(Bytes::from_static(b"Hello")))
        );
    }

    #[test]
    fn test_flushall_keys() {
        let storage = StorageEngine::with_capacity(100);
        storage.set(
            Bytes::from_static(b"key1"),
            RedisValue::String(Bytes::from_static(b"v1")),
            None,
        );

        // KEYS command
        let keys_cmd = resp_array(vec![b"KEYS", b"*"]);
        let cmd = Command::from_resp(keys_cmd).unwrap();
        let response = cmd.execute(&storage);
        println!("Actual response: {:?}", response);
        assert!(matches!(response, RespValue::Array(Some(_))));

        // FLUSHALL
        let flush_cmd = resp_array(vec![b"FLUSHALL"]);
        let cmd = Command::from_resp(flush_cmd).unwrap();
        let response = cmd.execute(&storage);
        assert_eq!(response, RespValue::SimpleString("OK".to_string()));
        assert!(storage.get(&Bytes::from_static(b"key1")).is_none());
    }

    #[test]
    fn test_ttl_expire_persist() {
        let storage = StorageEngine::with_capacity(100);
        let key = b"temp_key";
        storage.set(
            Bytes::from_static(key),
            RedisValue::String(Bytes::from_static(b"value")),
            None,
        );

        // EXPIRE
        let expire_cmd = resp_array(vec![b"EXPIRE", key, b"60"]);
        let cmd = Command::from_resp(expire_cmd).unwrap();
        let response = cmd.execute(&storage);
        assert_eq!(response, RespValue::Integer(1));

        // TTL should be positive
        let ttl_cmd = resp_array(vec![b"TTL", key]);
        let cmd = Command::from_resp(ttl_cmd).unwrap();
        let response = cmd.execute(&storage);
        assert!(response.as_integer().unwrap() > 0);

        // PERSIST
        let persist_cmd = resp_array(vec![b"PERSIST", key]);
        let cmd = Command::from_resp(persist_cmd).unwrap();
        let response = cmd.execute(&storage);
        assert_eq!(response, RespValue::Integer(1));

        // TTL should be -1 after persist
        let ttl_cmd = resp_array(vec![b"TTL", key]);
        let cmd = Command::from_resp(ttl_cmd).unwrap();
        let response = cmd.execute(&storage);
        assert_eq!(response, RespValue::Integer(-1));
    }

    #[test]
    fn test_list_commands() {
        let storage = StorageEngine::with_capacity(100);
        let key = b"mylist";

        // LPUSH
        let lpush_cmd = resp_array(vec![b"LPUSH", key, b"a", b"b"]);
        let cmd = Command::from_resp(lpush_cmd).unwrap();
        let response = cmd.execute(&storage);
        assert_eq!(response, RespValue::Integer(2));

        // RPUSH
        let rpush_cmd = resp_array(vec![b"RPUSH", key, b"c", b"d"]);
        let cmd = Command::from_resp(rpush_cmd).unwrap();
        let response = cmd.execute(&storage);
        assert_eq!(response, RespValue::Integer(4));

        // LLEN
        let llen_cmd = resp_array(vec![b"LLEN", key]);
        let cmd = Command::from_resp(llen_cmd).unwrap();
        let response = cmd.execute(&storage);
        assert_eq!(response, RespValue::Integer(4));

        // LPOP single
        let lpop_cmd = resp_array(vec![b"LPOP", key]);
        let cmd = Command::from_resp(lpop_cmd).unwrap();
        println!("{:?}", cmd);
        let response = cmd.execute(&storage);
        assert_eq!(
            response,
            RespValue::BulkString(Some(Bytes::from_static(b"b")))
        );

        // Debug to make sure everythings going alright
        let llen_cmd = resp_array(vec![b"LLEN", key]);
        let cmd = Command::from_resp(llen_cmd).unwrap();
        let response = cmd.execute(&storage);
        assert_eq!(response, RespValue::Integer(3)); // Should be left with acd

        // RPOP multiple
        let rpop_cmd = resp_array(vec![b"RPOP", key, b"2"]);
        let cmd = Command::from_resp(rpop_cmd).unwrap();
        let response = cmd.execute(&storage);
        if let RespValue::Array(Some(elements)) = response {
            assert_eq!(elements.len(), 2);
            assert_eq!(
                elements[0],
                RespValue::BulkString(Some(Bytes::from_static(b"d")))
            );
            assert_eq!(
                elements[1],
                RespValue::BulkString(Some(Bytes::from_static(b"c")))
            );
        } else {
            panic!("Expected array response");
        }

        // Final length should be 1
        let llen_cmd = resp_array(vec![b"LLEN", key]);
        let cmd = Command::from_resp(llen_cmd).unwrap();
        let response = cmd.execute(&storage);
        println!("response: {:?}", response);
        assert_eq!(response, RespValue::Integer(1));
    }

    #[test]
    fn test_type_errors() {
        let storage = StorageEngine::with_capacity(100);
        let key = b"key";

        // Set as string
        storage.set(
            Bytes::from_static(key),
            RedisValue::String(Bytes::from_static(b"value")),
            None,
        );

        // Try list operation on string
        let lpush_cmd = resp_array(vec![b"LPUSH", key, b"elem"]);
        let cmd = Command::from_resp(lpush_cmd).unwrap();
        let response = cmd.execute(&storage);
        println!("{:?}", response);
        assert!(response.is_error());
    }

    #[test]
    fn test_argument_errors() {
        // GET with no arguments
        let get_cmd = resp_array(vec![b"GET"]);
        assert!(matches!(
            Command::from_resp(get_cmd),
            Err(ParseError::InvalidArgCount {
                expected: 1,
                got: 0
            })
        ));

        // SET with insufficient arguments
        let set_cmd = resp_array(vec![b"SET", b"key"]);
        assert!(matches!(
            Command::from_resp(set_cmd),
            Err(ParseError::InvalidArgCount {
                expected: 2,
                got: 1
            })
        ));

        // EXPIRE with invalid duration
        let expire_cmd = resp_array(vec![b"EXPIRE", b"key", b"not_a_number"]);
        assert!(matches!(
            Command::from_resp(expire_cmd),
            Err(ParseError::InvalidDuration(_))
        ));
    }
}
