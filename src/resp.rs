use atoi::atoi;
use bytes::Bytes;
use memchr::memmem;
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

    pub fn is_error(&self) -> bool {
        self.is_err()
    }

    pub fn is_err(&self) -> bool {
        matches!(self, RespValue::Error(_))
    }

    /// Returns (end, start) around the \r\n, does not include \r\n.
    fn find_crlf(input: &Bytes, start: usize) -> Result<(usize, usize), ParseError> {
        memmem::find(&input[start..], b"\r\n")
            .map(|i| (start + i, start + i + 2))
            .ok_or(ParseError::Incomplete)
    }

    fn parse_simple_string(input: &Bytes, start: usize) -> Result<(RespValue, usize), ParseError> {
        let (end, next_start) = Self::find_crlf(input, start)?;
        String::from_utf8(input[start..end].into())
            .map(|s| (RespValue::SimpleString(s), next_start))
            .map_err(|_| ParseError::ByteError("Invalid UTF-8 in simple string".to_string()))
    }

    fn parse_integer(input: &Bytes, start: usize) -> Result<(RespValue, usize), ParseError> {
        let (end, next_start) = Self::find_crlf(input, start)?;
        atoi::<i64>(&input[start..end])
            .ok_or_else(|| {
                ParseError::NotAnInteger(String::from_utf8_lossy(&input[start..end]).into())
            })
            .map(|i| (RespValue::Integer(i), next_start))
    }

    fn parse_bulk_string(input: &Bytes, start: usize) -> Result<(RespValue, usize), ParseError> {
        let (end, next_start) = Self::find_crlf(input, start)?;
        let len = atoi::<i64>(&input[start..end]).ok_or_else(|| {
            ParseError::NotAnInteger(String::from_utf8_lossy(&input[start..end]).into())
        })?;

        match len {
            // null bulk string
            -1 => Ok((RespValue::BulkString(None), next_start)),
            len if len >= 0 => {
                let len = len as usize;
                let data_end = next_start + len;
                let crlf_end = data_end + 2;
                if input.len() < crlf_end {
                    Err(ParseError::Incomplete)
                } else if &input[data_end..crlf_end] != b"\r\n" {
                    Err(ParseError::ByteError(
                        "missing CRLF after bulk string".into(),
                    ))
                } else {
                    Ok((
                        RespValue::BulkString(Some(input.slice(next_start..data_end))),
                        crlf_end,
                    ))
                }
            }
            _ => Err(ParseError::LengthError(len)),
        }
    }

    fn parse_array(input: &Bytes, start: usize) -> Result<(RespValue, usize), ParseError> {
        let (end, mut next_start) = Self::find_crlf(input, start)?;
        let len = atoi::<i64>(&input[start..end]).ok_or_else(|| {
            ParseError::NotAnInteger(String::from_utf8_lossy(&input[start..end]).into())
        })?;

        match len {
            // null
            -1 => Ok((RespValue::Array(None), next_start)),
            // empty
            0 => Ok((RespValue::Array(Some(vec![])), next_start)),
            // Standard array
            len if len > 0 => {
                let mut items = Vec::with_capacity(len as usize);
                for _ in 0..len {
                    let (item, remaining_start_index) = Self::parse_at(input, next_start)?;
                    items.push(item);
                    next_start = remaining_start_index;
                }
                Ok((RespValue::Array(Some(items)), next_start))
            }
            len => Err(ParseError::LengthError(len)),
        }
    }

    fn parse_error(input: &Bytes, start: usize) -> Result<(RespValue, usize), ParseError> {
        let (end, next_start) = Self::find_crlf(input, start)?;
        String::from_utf8(input[start..end].into())
            .map(|s| (RespValue::Error(s), next_start))
            .map_err(|_| ParseError::ByteError("Invalid UTF-8 in error message".to_string()))
    }

    fn parse_inline(input: &Bytes, start: usize) -> Result<(RespValue, usize), ParseError> {
        let (end, next_start) = Self::find_crlf(input, start)?;
        let s: Bytes = input.slice(start..end);
        if s.contains(&b' ') {
            return Err(ParseError::ByteError(
                format!(
                    "Inline command contains a space: {}",
                    str::from_utf8(&s).unwrap_or("Error parsing command")
                )
                .to_string(),
            ));
        }
        Ok((
            RespValue::Array(vec![RespValue::BulkString(Some(s))].into()),
            next_start,
        ))
    }

    fn parse_at(input: &Bytes, start: usize) -> Result<(RespValue, usize), ParseError> {
        if start >= input.len() {
            return Err(ParseError::Incomplete);
        }

        match input[start] {
            b'+' => Self::parse_simple_string(input, start + 1),
            b'-' => Self::parse_error(input, start + 1),
            b':' => Self::parse_integer(input, start + 1),
            b'$' => Self::parse_bulk_string(input, start + 1),
            b'*' => Self::parse_array(input, start + 1),
            _ => Self::parse_inline(input, start),
        }
    }

    pub fn parse(input: &Bytes) -> Result<(RespValue, usize), ParseError> {
        Self::parse_at(input, 0)
    }

    pub fn serialize(self) -> Vec<u8> {
        match self {
            RespValue::SimpleString(s) => Self::serialize_simple_string(s),
            RespValue::BulkString(opt) => Self::serialize_bulk_string(opt),
            RespValue::Array(opt) => Self::serialize_array(opt),
            RespValue::Integer(i) => Self::serialize_integer(i),
            RespValue::Error(e) => Self::serialize_error(e),
        }
    }

    fn serialize_simple_string(s: String) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(3 + s.len());
        bytes.push(b'+');
        bytes.extend(Bytes::from(s));
        bytes.extend(b"\r\n");
        bytes
    }

    fn serialize_error(e: String) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(3 + e.len());
        bytes.push(b'-');
        bytes.extend(Bytes::from(e));
        bytes.extend(b"\r\n");
        bytes
    }

    fn serialize_integer(i: i64) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.push(b':');
        bytes.extend(Bytes::from(i.to_string()));
        bytes.extend(b"\r\n");
        bytes
    }

    fn serialize_bulk_string(s: Option<Bytes>) -> Vec<u8> {
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

    fn serialize_array(opt: Option<Vec<RespValue>>) -> Vec<u8> {
        match opt {
            Some(elements) => {
                let mut bytes: Vec<u8> = Vec::new();
                bytes.push(b'*');
                bytes.extend(Bytes::from(elements.len().to_string()));
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
        let input = &Bytes::from_static(b"+OK\r\n");
        let (result, _) = RespValue::parse(input).unwrap();
        assert_eq!(result, RespValue::SimpleString("OK".to_string()));
    }

    #[test]
    fn test_error() {
        let input = &Bytes::from_static(b"-ERR unknown command\r\n");
        let (result, _) = RespValue::parse(input).unwrap();
        assert_eq!(result, RespValue::Error("ERR unknown command".to_string()));
    }

    #[test]
    fn test_integer() {
        let input = &Bytes::from_static(b":1000\r\n");
        let (result, _) = RespValue::parse(input).unwrap();
        assert_eq!(result, RespValue::Integer(1000));
    }

    #[test]
    fn test_integer_zero() {
        let input = &Bytes::from_static(b":0\r\n");
        let (result, _) = RespValue::parse(input).unwrap();
        assert_eq!(result, RespValue::Integer(0));
    }

    #[test]
    fn test_integer_negative() {
        let input = &Bytes::from_static(b":-42\r\n");
        let (result, _) = RespValue::parse(input).unwrap();
        assert_eq!(result, RespValue::Integer(-42));
    }

    #[test]
    fn test_bulk_string() {
        let input = &Bytes::from_static(b"$5\r\nhello\r\n");
        let (result, _) = RespValue::parse(input).unwrap();
        assert_eq!(
            result,
            RespValue::BulkString(Option::from(Bytes::from_static(b"hello")))
        );
    }

    #[test]
    fn test_bulk_string_empty() {
        let input = &Bytes::from_static(b"$0\r\n\r\n");
        let (result, _) = RespValue::parse(input).unwrap();
        assert_eq!(
            result,
            RespValue::BulkString(Option::from(Bytes::from_static(b"")))
        );
    }

    #[test]
    fn test_bulk_string_null() {
        let input = &Bytes::from_static(b"$-1\r\n");
        let (result, _) = RespValue::parse(input).unwrap();
        assert_eq!(result, RespValue::BulkString(None));
    }

    #[test]
    fn test_array() {
        let input = &Bytes::from_static(b"*2\r\n$3\r\nfoo\r\n$3\r\nbar\r\n");
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
        let input = &Bytes::from_static(b"*0\r\n");
        let (result, _) = RespValue::parse(input).unwrap();
        assert_eq!(result, RespValue::Array(vec![].into()));
    }

    #[test]
    fn test_array_null() {
        let input = &Bytes::from_static(b"*-1\r\n");
        let (result, _) = RespValue::parse(input).unwrap();
        assert_eq!(result, RespValue::Array(None));
    }

    #[test]
    fn test_array_mixed_types() {
        let input = &Bytes::from_static(b"*3\r\n:1\r\n+OK\r\n$-1\r\n");
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
        let input = &Bytes::from_static(b"*2\r\n*1\r\n:1\r\n+OK\r\n");
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
    fn test_invalid_inline() {
        let input = &Bytes::from_static(b"PING Hi\r\n");
        let result = RespValue::parse(input);
        assert!(result.is_err());
    }

    #[test]
    fn test_incomplete_simple_string() {
        let input = &Bytes::from_static(b"+OK");
        let result = RespValue::parse(input);
        println!("{:?}", result);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_bulk_string_length() {
        let input = &Bytes::from_static(b"$abc\r\nhello\r\n");
        let result = RespValue::parse(input);
        assert!(result.is_err());
    }

    #[test]
    fn test_negative_bulk_string_length() {
        let input = &Bytes::from_static(b"$-2\r\n");
        let result = RespValue::parse(input);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_array_length() {
        let input = &Bytes::from_static(b"*abc\r\n");
        let result = RespValue::parse(input);
        assert!(result.is_err());
    }

    #[test]
    fn test_negative_array_length() {
        let input = &Bytes::from_static(b"*-2\r\n");
        let result = RespValue::parse(input);
        assert!(result.is_err());
    }

    #[test]
    fn test_incomplete_array() {
        let input = &Bytes::from_static(b"*2\r\n:1\r\n");
        let result = RespValue::parse(input);
        assert!(result.is_err());
    }

    #[test]
    fn test_ok_inline_array() {
        let input = &Bytes::from_static(b"PING\r\n");
        let result = RespValue::parse(input);
        assert!(result.is_ok());
        let (inline_ping, _) = result.unwrap();
        assert_eq!(
            inline_ping,
            RespValue::Array(vec![RespValue::BulkString(Some(Bytes::from("PING")))].into())
        );
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
