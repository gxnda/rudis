use crate::storage::memory::{DataEntry, RedisValue, StorageEngine};
use crate::{resp::RespValue, storage::memory::IncrError};
use bytes::Bytes;
use std::collections::VecDeque;
use std::time::{Duration, Instant};

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

#[derive(Debug)]
pub enum ParseError {
    InvalidCommand(String),
    InvalidArgCount { expected: usize, got: usize },
    InvalidArgument(String),
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
                    let count_str = std::str::from_utf8(&arg_bytes[1])
                        .map_err(|_| ParseError::InvalidArgument("Invalid count".to_string()))?;
                    let count = count_str
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
                        RespValue::SimpleString(
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
                        RespValue::SimpleString(
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
                Some(RedisValue::List(mut arr)) => match count {
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
                    None => RespValue::BulkString(arr.pop_front()),
                },
                _ => RespValue::BulkString(None),
            },

            Command::RPop { key, count } => match storage.get(key) {
                Some(RedisValue::List(mut arr)) => match count {
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
                    None => RespValue::BulkString(arr.pop_front()),
                },
                _ => RespValue::BulkString(None),
            },

            Command::LLen { key } => match storage.get(key) {
                Some(RedisValue::List(arr)) => RespValue::Integer(arr.len() as i64),
                None => RespValue::Integer(0),
                _ => RespValue::Error(
                    "WRONGTYPE Operation against a key holding the wrong kind of value".to_string(),
                ),
            },

            _ => RespValue::Error("Server error, command unknown.".to_string()),
        }
    }
}

#[cfg(test)]
mod command_tests {
    use super::*;
    use std::time::Duration;

    fn create_resp_array(items: Vec<&str>) -> RespValue {
        let mut array = Vec::new();
        for item in items {
            array.push(RespValue::BulkString(Some(Bytes::copy_from_slice(
                item.as_bytes(),
            ))));
        }
        RespValue::Array(Some(array))
    }

    #[test]
    fn test_get_command() {
        let resp = create_resp_array(vec!["GET", "mykey"]);
        let cmd = Command::from_resp(resp).unwrap();
        assert!(matches!(cmd, Command::Get { key } if key == "mykey"));
    }

    #[test]
    fn test_get_command_invalid() {
        let resp = create_resp_array(vec!["GET"]);
        let result = Command::from_resp(resp);
        assert!(matches!(
            result,
            Err(ParseError::InvalidArgCount {
                expected: 1,
                got: 0
            })
        ));

        let resp = create_resp_array(vec!["GET", "key1", "key2"]);
        let result = Command::from_resp(resp);
        assert!(matches!(
            result,
            Err(ParseError::InvalidArgCount {
                expected: 1,
                got: 2
            })
        ));
    }

    #[test]
    fn test_set_command() {
        let resp = create_resp_array(vec!["SET", "key", "value"]);
        let cmd = Command::from_resp(resp).unwrap();
        assert!(matches!(
            cmd,
            Command::Set {
                key,
                value,
                ttl: None
            } if key == "key" && value == "value"
        ));
    }

    #[test]
    fn test_set_command_with_ttl() {
        let resp = create_resp_array(vec!["SET", "key", "value", "EX", "10"]);
        let cmd = Command::from_resp(resp).unwrap();
        assert!(matches!(
            cmd,
            Command::Set {
                key,
                value,
                ttl: Some(ttl)
            } if key == "key" && value == "value" && ttl > Instant::now()
        ));

        let resp = create_resp_array(vec!["SET", "key", "value", "PX", "500"]);
        let cmd = Command::from_resp(resp).unwrap();
        assert!(matches!(
            cmd,
            Command::Set {
                ttl: Some(ttl),
                ..
            } if ttl > Instant::now()
        ));

        // Alternative approach with more precise testing
        assert!(matches!(
            cmd,
            Command::Set {
                key,
                value,
                ttl: Some(ttl)
            } if key == "key" && value == "value" && ttl > Instant::now()
        ));

        let resp = create_resp_array(vec!["SET", "key", "value", "PX", "500"]);
        let cmd = Command::from_resp(resp).unwrap();
        assert!(matches!(
            cmd,
            Command::Set {
                ttl: Some(ttl),
                ..
            } if ttl > Instant::now()
        ));

        // Alternative approach with more precise testing
        let before = Instant::now();
        let resp = create_resp_array(vec!["SET", "key", "value", "EX", "10"]);
        let cmd = Command::from_resp(resp).unwrap();
        let after = Instant::now();
        assert!(matches!(
            cmd,
            Command::Set {
                key,
                value,
                ttl: Some(ttl)
            } if key == "key"
                && value == "value"
                && ttl >= before + Duration::from_secs(10)
                && ttl <= after + Duration::from_secs(10)
        ));

        let before = Instant::now();
        let resp = create_resp_array(vec!["SET", "key", "value", "PX", "500"]);
        let cmd = Command::from_resp(resp).unwrap();
        let after = Instant::now();
        assert!(matches!(
            cmd,
            Command::Set {
                ttl: Some(ttl),
                ..
            } if ttl >= before + Duration::from_millis(500)
                && ttl <= after + Duration::from_millis(500)
        ));
    }

    #[test]
    fn test_set_command_invalid() {
        let resp = create_resp_array(vec!["SET", "key"]);
        let result = Command::from_resp(resp);
        assert!(matches!(
            result,
            Err(ParseError::InvalidArgCount {
                expected: 2,
                got: 1
            })
        ));

        let resp = create_resp_array(vec!["SET", "key", "value", "INVALID", "10"]);
        let result = Command::from_resp(resp);
        assert!(matches!(result, Err(ParseError::InvalidArgument(_))));

        let resp = create_resp_array(vec!["SET", "key", "value", "EX", "invalid"]);
        let result = Command::from_resp(resp);
        assert!(matches!(result, Err(ParseError::InvalidDuration(_))));
    }

    #[test]
    fn test_del_command() {
        let resp = create_resp_array(vec!["DEL", "key1", "key2", "key3"]);
        let cmd = Command::from_resp(resp).unwrap();
        assert!(matches!(cmd, Command::Del { keys } if keys.len() == 3));
    }

    #[test]
    fn test_ping_command() {
        let resp = create_resp_array(vec!["PING"]);
        let cmd = Command::from_resp(resp).unwrap();
        assert!(matches!(cmd, Command::Ping { message: None }));

        let resp = create_resp_array(vec!["PING", "Hello"]);
        let cmd = Command::from_resp(resp).unwrap();
        assert!(matches!(cmd, Command::Ping { message: Some(msg) } if msg == "Hello"));
    }

    #[test]
    fn test_expire_command() {
        let resp = create_resp_array(vec!["EXPIRE", "mykey", "10"]);
        let cmd = Command::from_resp(resp).unwrap();
        assert!(matches!(
            cmd,
            Command::Expire { key, ttl }
            if key == "mykey" && ttl == Duration::from_secs(10)
        ));
    }

    #[test]
    fn test_list_commands() {
        let resp = create_resp_array(vec!["LPUSH", "mylist", "v1", "v2", "v3"]);
        let cmd = Command::from_resp(resp).unwrap();
        assert!(matches!(cmd, Command::LLen { key } if key == "mylist"));
    }

    #[test]
    fn test_invalid_commands() {
        let resp = RespValue::SimpleString("PING".to_string());
        let result = Command::from_resp(resp);
        assert!(matches!(result, Err(ParseError::InvalidCommand(_))));

        let resp = RespValue::Array(Some(vec![
            RespValue::Integer(42),
            RespValue::BulkString(Some(Bytes::from_static(b"key"))),
        ]));
        let result = Command::from_resp(resp);
        assert!(matches!(result, Err(ParseError::InvalidCommand(_))));

        let resp: RespValue = RespValue::Array(Some(vec![RespValue::BulkString(Some(
            Bytes::from_static(b"INVALID"),
        ))]));
        let result = Command::from_resp(resp);
        assert!(matches!(result, Err(ParseError::InvalidCommand(_))));
    }
}
