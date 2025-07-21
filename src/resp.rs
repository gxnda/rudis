
// Parser for redis RESP

// Summarised from https://github.com/redis/redis-specifications/blob/master/protocol/RESP2.md

use std::slice::Iter;

#[derive(Debug)]
pub enum RespValue {
    SimpleString(String),
    BulkString(String),
    Array(Vec<RespValue>),
    Integer(i64),
    Error(String),
    Null,
}

impl RespValue {
    fn parse_str(chars: &mut Iter<u8>) -> Result<String, &'static str> {
        // Simple String (+): +<str>\r\n
        // - "+OK\r\n"
        // - Followed by a char that is not CR or LF, terminated by CRLF
        // - Not binary safe (only ASCII?)
        let mut bytes: Vec<u8> = Vec::new();
        loop {
            match chars.next() {
                Some(&b'\r') => {
                    match chars.next() {
                        Some(&b'\n') => break,
                        _ => return Err("Expected CRLF after a simple string"),
                    }
                }
                Some(b) => bytes.push(*b),
                None => return Err("Unexpected end of input"),
            }
        }
        let s =std::str::from_utf8(&bytes)
            .map_err(|_| "Invalid UTF-8 in simple string")?;
        Ok(s.to_string())
    }
    fn parse_int(chars: &mut Iter<u8>) -> Result<Option<i64>, &'static str> {
        let s: String = Self::parse_str(chars)?;
        if s.parse::<i64>().is_ok() {
            let i: i64 = s.parse::<i64>().unwrap();
            if i == -1 {
                Ok(None) // Null value
            } else if i >= 0 {
                Ok(Some(i))
            } else {
                Err("Invalid integer")
            }
        } else { Err("Invalid integer") }
    }

    fn parse_bulk_string(chars: &mut Iter<u8>) -> Result<Option<String>, &'static str> {
        let len = match Self::parse_int(chars)? {
            Some(-1) => return Ok(None),
            Some(l) if l >= 0 => l as usize,
            _ => return Err("Invalid bulk string length"),
        };
        let mut bytes = Vec::with_capacity(len);
        for _ in 0..len {
            bytes.push(*chars.next().ok_or("Unexpected end of bulk string data")?);
        }
        if chars.next() != Some(&b'\r') || chars.next() != Some(&b'\n') {
            return Err("Bulk string isn't terminated with CRLF");
        }
        String::from_utf8(bytes)
            .map(Some)
            .map_err(|_| "Invalid UTF-8 in bulk string")
    }

    fn parse_array(chars: &mut Iter<u8>) -> Result<Option<Vec<RespValue>>, &'static str> {
        // RESP Arrays (*): *<len>CRLF<elems>
        // - int for number of elements
        // - "*0\r\n" for empty
        // - "*2\r\n$3\r\nfoo\r\n$3\r\nbar\r\n"
        // - can be mixed type arrays
        // - null array "*-1\r\n"
        // - null elements indicate missing elements
        let size = match Self::parse_int(chars)? {
            Some(-1) => return Ok(None),
            Some(0) => return Ok(Some(Vec::new())),
            Some(l) if l >= 0 => l as usize,
            _ => return Err("Invalid bulk string length"),
        };
        let mut array: Vec<RespValue> = Vec::with_capacity(size);
        for _ in 0..size {
            array.push(Self::parse_iter(chars)?);
        }
        Ok(Some(array))
    }

    pub fn parse(input: &[u8]) -> Result<RespValue, &'static str> {
        if input.is_empty() {
            return Err("Empty input")
        }
        let mut iter: Iter<u8> = input.iter();
        Self::parse_iter(&mut iter)
    }

    fn parse_iter(chars: &mut Iter<u8>) -> Result<RespValue, &'static str> {
        let first_char= chars.next().ok_or("Unexpected end of input")?;
        match first_char {

            // Simple String (+): +<str>\r\n
            // - "+OK\r\n"
            // - Followed by a char that is not CR or LF, terminated by CRLF
            b'+' => { Self::parse_str(chars).map(RespValue::SimpleString) }

            // RESP Errors (-): -<str>\r\n
            // - "-ERR unknown command 'foobar\r\n"
            // - Same as simple strings, just with a minus instead.
            // - Error prefixes exist (such as "ERR", "WRONGTYPE", var len)
            b'-' => { Self::parse_str(chars).map(RespValue::Error) }

            // RESP Integers (:): :<num>\r\n
            // - ":0\r\n"
            // - The number is just a string which gets read
            // - Within range of i64
            // - Often used as bools (0/1), or as an indication of function performing
            b':' => { Self::parse_int(chars).map(|opt| match opt {
                Some(i) => RespValue::Integer(i),
                None => RespValue::Null,
            }) }

            b'$' => {
                Self::parse_bulk_string(chars).map(|opt| match opt {
                    Some(s) => RespValue::BulkString(s),
                    None => RespValue::Null,
                })
            }

            b'*' => {
                // RESP Arrays (*): *<len>CRLF<elems>
                // - int for number of elements
                // - "*0\r\n" for empty
                // - "*2\r\n$3\r\nfoo\r\n$3\r\nbar\r\n"
                // - can be mixed type arrays
                // - null array "*-1\r\n"
                // - null elements indicate missing elements
                Self::parse_array(chars).map(|opt| match opt {
                    Some(a) => RespValue::Array(a),
                    None => RespValue::Null,
                })
            }
            _ => {
                Err("Invalid first character")
            }
        }
    }

}