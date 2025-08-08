use bytes::Bytes;
use std::array::TryFromSliceError;
use thiserror::Error;

#[derive(Debug, PartialEq)]
pub enum RespValue {
    SimpleString(String),
    BulkString(Option<Bytes>),
    Array(Option<Vec<RespValue>>),
    Integer(i64),
    Error(String),
}

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("Incomplete parse")]
    Incomplete,
    #[error("Not an integer: {0}")]
    NotAnInteger(String),
    #[error("Error parsing bytes: {0}")]
    ByteError(String),
    #[error("Invalid length: {0}")]
    LengthError(i64),
}

impl RespValue {
    pub fn as_integer(&self) -> Result<i64, ParseError> {
        match self {
            RespValue::Integer(i) => Ok(*i),
            _ => Err(ParseError::NotAnInteger("".to_string())),
        }
    }

    pub fn force_as_integer(&self) -> Result<i64, ParseError> {
        // like as_integer, but forces Strings into int if they are numeric.
        match self {
            RespValue::Integer(i) => Ok(*i),
            RespValue::BulkString(Some(bytes)) => {
                let array: [u8; 8] = bytes
                    .as_ref()
                    .try_into()
                    .map_err(|e: TryFromSliceError| ParseError::ByteError(e.to_string()))?;
                Ok(i64::from_be_bytes(array))
            }
            _ => Err(ParseError::NotAnInteger("".to_string())),
        }
    }

    pub fn is_error(&self) -> bool {
        self.is_err()
    }

    pub fn is_err(&self) -> bool {
        matches!(self, RespValue::Error(_))
    }

    fn parse_until_crlf(input: &[u8]) -> Result<(&[u8], &[u8]), ParseError> {
        let mut end = 0;
        while end + 1 < input.len() {
            if input[end] == b'\r' && input[end + 1] == b'\n' {
                return Ok((&input[..end], &input[end + 2..]));
            }
            end += 1;
        }
        Err(ParseError::Incomplete)
    }

    fn parse_simple_string(input: &[u8]) -> Result<(RespValue, &[u8]), ParseError> {
        Self::parse_until_crlf(input).and_then(|(s, rest)| {
            String::from_utf8(s.to_vec())
                .map(|s| (RespValue::SimpleString(s), rest))
                .map_err(|e| ParseError::ByteError(e.to_string()))
        })
    }

    fn parse_integer(input: &[u8]) -> Result<(RespValue, &[u8]), ParseError> {
        Self::parse_until_crlf(input).and_then(|(num_bytes, rest)| {
            std::str::from_utf8(num_bytes)
                .map_err(|e| ParseError::ByteError(e.to_string()))
                .and_then(|s| {
                    s.parse::<i64>()
                        .map(|i| (RespValue::Integer(i), rest))
                        .map_err(|e| ParseError::ByteError(e.to_string()))
                })
        })
    }

    fn parse_bulk_string(input: &[u8]) -> Result<(RespValue, &[u8]), ParseError> {
        let (len_bytes, rest) = Self::parse_until_crlf(input)?;
        let len = std::str::from_utf8(len_bytes)
            .map_err(|e| ParseError::ByteError(e.to_string()))?
            .parse::<i64>()
            .map_err(|e| ParseError::NotAnInteger(e.to_string()))?;

        match len {
            // null bulk string
            -1 => Ok((RespValue::BulkString(None), rest)),
            // empty
            0 => {
                if rest.len() < 2 || &rest[..2] != b"\r\n" {
                    Err(ParseError::Incomplete)
                } else {
                    Ok((RespValue::BulkString(Some(Bytes::new())), &rest[2..]))
                }
            }
            len if len > 0 => {
                let len = len as usize;
                if rest.len() < len + 2 {
                    Err(ParseError::Incomplete)
                } else if &rest[len..len + 2] != b"\r\n" {
                    Err(ParseError::Incomplete)
                } else {
                    let data = Bytes::copy_from_slice(&rest[..len]);
                    Ok((RespValue::BulkString(Some(data)), &rest[len + 2..]))
                }
            }
            _ => Err(ParseError::LengthError(len)),
        }
    }

    fn parse_array(chars: &[u8]) -> Result<(RespValue, &[u8]), ParseError> {
        let (len_bytes, mut rest) = Self::parse_until_crlf(chars)?;
        let len_str =
            std::str::from_utf8(len_bytes).map_err(|e| ParseError::ByteError(e.to_string()))?;
        let len = len_str
            .parse::<i64>()
            .map_err(|e| ParseError::NotAnInteger(e.to_string()))?;

        match len {
            // null
            -1 => Ok((RespValue::Array(None), rest)),
            // empty
            0 => Ok((RespValue::Array(Some(vec![])), rest)),
            // Standard array
            len if len > 0 => {
                let mut items = Vec::with_capacity(len as usize);
                for _ in 0..len {
                    let (item, remaining) = Self::parse(rest)?;
                    items.push(item);
                    rest = remaining;
                }
                Ok((RespValue::Array(Some(items)), rest))
            }
            len => Err(ParseError::LengthError(len)),
        }
    }

    fn parse_error(input: &[u8]) -> Result<(RespValue, &[u8]), ParseError> {
        Self::parse_until_crlf(input).and_then(|(s, rest)| {
            String::from_utf8(s.to_vec())
                .map(|s| (RespValue::Error(s), rest))
                .map_err(|_| ParseError::ByteError("Invalid UTF-8 in error message".to_string()))
        })
    }

    pub fn parse(input: &[u8]) -> Result<(RespValue, &[u8]), ParseError> {
        if input.is_empty() {
            return Err(ParseError::Incomplete);
        }

        match input[0] {
            b'+' => Self::parse_simple_string(&input[1..]),
            b'-' => Self::parse_error(&input[1..]),
            b':' => Self::parse_integer(&input[1..]),
            b'$' => Self::parse_bulk_string(&input[1..]),
            b'*' => Self::parse_array(&input[1..]),
            _ => Err(ParseError::ByteError(
                "Invalid prefix in RESP input".to_string(),
            )),
        }
    }

    pub fn serialize(&self) -> Vec<u8> {
        match self {
            RespValue::SimpleString(s) => Self::serialize_simple_string(s),
            RespValue::BulkString(opt) => Self::serialize_bulk_string(opt.as_ref()),
            RespValue::Array(opt) => Self::serialize_array(opt.as_deref()),
            RespValue::Integer(i) => Self::serialize_integer(*i),
            RespValue::Error(e) => Self::serialize_error(e),
        }
    }

    fn serialize_simple_string(s: &str) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(3 + s.len());
        bytes.push(b'+');
        bytes.extend(s.as_bytes());
        bytes.extend(b"\r\n");
        bytes
    }

    fn serialize_error(e: &str) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(3 + e.len());
        bytes.push(b'-');
        bytes.extend(e.as_bytes());
        bytes.extend(b"\r\n");
        bytes
    }

    fn serialize_integer(i: i64) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.push(b':');
        bytes.extend(i.to_string().as_bytes());
        bytes.extend(b"\r\n");
        bytes
    }

    fn serialize_bulk_string(s: Option<&Bytes>) -> Vec<u8> {
        match s {
            Some(s) => {
                let mut bytes = Vec::new();
                bytes.push(b'$');
                bytes.extend(s.len().to_string().as_bytes());
                bytes.extend(b"\r\n");
                bytes.extend(s);
                bytes.extend(b"\r\n");
                bytes
            }
            None => b"$-1\r\n".to_vec(),
        }
    }

    fn serialize_array(opt: Option<&[RespValue]>) -> Vec<u8> {
        match opt {
            Some(elements) => {
                let mut bytes = Vec::new();
                bytes.push(b'*');
                bytes.extend(elements.len().to_string().as_bytes());
                bytes.extend(b"\r\n");
                for elem in elements {
                    bytes.extend(elem.serialize());
                }
                bytes
            }
            None => b"*-1\r\n".to_vec(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_string() {
        let input = b"+OK\r\n";
        let (result, _) = RespValue::parse(input).unwrap();
        assert_eq!(result, RespValue::SimpleString("OK".to_string()));
    }

    #[test]
    fn test_error() {
        let input = b"-ERR unknown command\r\n";
        let (result, _) = RespValue::parse(input).unwrap();
        assert_eq!(result, RespValue::Error("ERR unknown command".to_string()));
    }

    #[test]
    fn test_integer() {
        let input = b":1000\r\n";
        let (result, _) = RespValue::parse(input).unwrap();
        assert_eq!(result, RespValue::Integer(1000));
    }

    #[test]
    fn test_integer_zero() {
        let input = b":0\r\n";
        let (result, _) = RespValue::parse(input).unwrap();
        assert_eq!(result, RespValue::Integer(0));
    }

    #[test]
    fn test_integer_negative() {
        let input = b":-42\r\n";
        let (result, _) = RespValue::parse(input).unwrap();
        assert_eq!(result, RespValue::Integer(-42));
    }

    #[test]
    fn test_bulk_string() {
        let input = b"$5\r\nhello\r\n";
        let (result, _) = RespValue::parse(input).unwrap();
        assert_eq!(
            result,
            RespValue::BulkString(Option::from(Bytes::from_static(b"hello")))
        );
    }

    #[test]
    fn test_bulk_string_empty() {
        let input = b"$0\r\n\r\n";
        let (result, _) = RespValue::parse(input).unwrap();
        assert_eq!(
            result,
            RespValue::BulkString(Option::from(Bytes::from_static(b"")))
        );
    }

    #[test]
    fn test_bulk_string_null() {
        let input = b"$-1\r\n";
        let (result, _) = RespValue::parse(input).unwrap();
        assert_eq!(result, RespValue::BulkString(None));
    }

    #[test]
    fn test_array() {
        let input = b"*2\r\n$3\r\nfoo\r\n$3\r\nbar\r\n";
        let (result, _) = RespValue::parse(input).unwrap();
        assert_eq!(
            result,
            RespValue::Array(Option::from(vec![
                RespValue::BulkString(Option::from(Bytes::from_static(b"foo"))),
                RespValue::BulkString(Option::from(Bytes::from_static(b"bar")))
            ]))
        );
    }

    #[test]
    fn test_array_empty() {
        let input = b"*0\r\n";
        let (result, _) = RespValue::parse(input).unwrap();
        assert_eq!(result, RespValue::Array(vec![].into()));
    }

    #[test]
    fn test_array_null() {
        let input = b"*-1\r\n";
        let (result, _) = RespValue::parse(input).unwrap();
        assert_eq!(result, RespValue::Array(None));
    }

    #[test]
    fn test_array_mixed_types() {
        let input = b"*3\r\n:1\r\n+OK\r\n$-1\r\n";
        let (result, _) = RespValue::parse(input).unwrap();
        assert_eq!(
            result,
            RespValue::Array(Option::from(vec![
                RespValue::Integer(1),
                RespValue::SimpleString("OK".to_string()),
                RespValue::BulkString(None)
            ]))
        );
    }

    #[test]
    fn test_nested_array() {
        let input = b"*2\r\n*1\r\n:1\r\n+OK\r\n";
        let (result, _) = RespValue::parse(input).unwrap();
        assert_eq!(
            result,
            RespValue::Array(Option::from(vec![
                RespValue::Array(Option::from(vec![RespValue::Integer(1)])),
                RespValue::SimpleString("OK".to_string())
            ]))
        );
    }

    #[test]
    fn test_invalid_start_byte() {
        let input = b"xinvalid\r\n";
        let result = RespValue::parse(input);
        assert!(result.is_err());
    }

    #[test]
    fn test_incomplete_simple_string() {
        let input = b"+OK";
        let result = RespValue::parse(input);
        println!("{:?}", result);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_bulk_string_length() {
        let input = b"$abc\r\nhello\r\n";
        let result = RespValue::parse(input);
        assert!(result.is_err());
    }

    #[test]
    fn test_negative_bulk_string_length() {
        let input = b"$-2\r\n";
        let result = RespValue::parse(input);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_array_length() {
        let input = b"*abc\r\n";
        let result = RespValue::parse(input);
        assert!(result.is_err());
    }

    #[test]
    fn test_negative_array_length() {
        let input = b"*-2\r\n";
        let result = RespValue::parse(input);
        assert!(result.is_err());
    }

    #[test]
    fn test_incomplete_array() {
        let input = b"*2\r\n:1\r\n";
        let result = RespValue::parse(input);
        assert!(result.is_err());
    }

    #[test]
    fn test_serialize_simple_string() {
        let value = RespValue::SimpleString("OK".to_string());
        assert_eq!(value.serialize(), b"+OK\r\n");
    }

    #[test]
    fn test_serialize_error() {
        let value = RespValue::Error("ERR unknown command".to_string());
        assert_eq!(value.serialize(), b"-ERR unknown command\r\n");
    }

    #[test]
    fn test_serialize_integer_positive() {
        let value = RespValue::Integer(1000);
        assert_eq!(value.serialize(), b":1000\r\n");
    }

    #[test]
    fn test_serialize_integer_zero() {
        let value = RespValue::Integer(0);
        assert_eq!(value.serialize(), b":0\r\n");
    }

    #[test]
    fn test_serialize_integer_negative() {
        let value = RespValue::Integer(-42);
        assert_eq!(value.serialize(), b":-42\r\n");
    }

    #[test]
    fn test_serialize_bulk_string() {
        let value = RespValue::BulkString(Some(Bytes::from_static(b"hello")));
        assert_eq!(value.serialize(), b"$5\r\nhello\r\n");
    }

    #[test]
    fn test_serialize_bulk_string_empty() {
        let value = RespValue::BulkString(Some(Bytes::from_static(b"")));
        assert_eq!(value.serialize(), b"$0\r\n\r\n");
    }

    #[test]
    fn test_serialize_bulk_string_null() {
        let value = RespValue::BulkString(None);
        assert_eq!(value.serialize(), b"$-1\r\n");
    }

    #[test]
    fn test_serialize_array() {
        let value = RespValue::Array(Some(vec![
            RespValue::BulkString(Some(Bytes::from_static(b"foo"))),
            RespValue::BulkString(Some(Bytes::from_static(b"bar"))),
        ]));
        assert_eq!(value.serialize(), b"*2\r\n$3\r\nfoo\r\n$3\r\nbar\r\n");
    }

    #[test]
    fn test_serialize_array_empty() {
        let value = RespValue::Array(Some(vec![]));
        assert_eq!(value.serialize(), b"*0\r\n");
    }

    #[test]
    fn test_serialize_array_null() {
        let value = RespValue::Array(None);
        assert_eq!(value.serialize(), b"*-1\r\n");
    }

    #[test]
    fn test_serialize_array_mixed_types() {
        let value = RespValue::Array(Some(vec![
            RespValue::Integer(1),
            RespValue::SimpleString("OK".to_string()),
            RespValue::BulkString(None),
        ]));
        assert_eq!(value.serialize(), b"*3\r\n:1\r\n+OK\r\n$-1\r\n");
    }

    #[test]
    fn test_serialize_nested_array() {
        let value = RespValue::Array(Some(vec![
            RespValue::Array(Some(vec![RespValue::Integer(1)])),
            RespValue::SimpleString("OK".to_string()),
        ]));
        assert_eq!(value.serialize(), b"*2\r\n*1\r\n:1\r\n+OK\r\n");
    }
}
