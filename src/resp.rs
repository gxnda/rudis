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

    fn find_crlf_u8(input: &[u8], start: usize) -> Result<(usize, usize), ParseError> {
        memmem::find(&input[start..], b"\r\n")
            .map(|i| (start + i, start + i + 2))
            .ok_or(ParseError::Incomplete)
    }

    fn parse_checked_simple_string(
        input: &Bytes,
        start: usize,
    ) -> Result<(RespValue, usize), ParseError> {
        let (end, next_start) = Self::find_crlf(input, start)?;
        String::from_utf8(input[start..end].into())
            .map(|s| (RespValue::SimpleString(s), next_start))
            .map_err(|_| ParseError::ByteError("Invalid UTF-8 in simple string".to_string()))
    }

    fn parse_checked_integer(
        input: &Bytes,
        start: usize,
    ) -> Result<(RespValue, usize), ParseError> {
        let (end, next_start) = Self::find_crlf(input, start)?;
        atoi::<i64>(&input[start..end])
            .ok_or_else(|| {
                ParseError::NotAnInteger(String::from_utf8_lossy(&input[start..end]).into())
            })
            .map(|i| (RespValue::Integer(i), next_start))
    }

    fn parse_checked_bulk_string(
        input: &Bytes,
        start: usize,
    ) -> Result<(RespValue, usize), ParseError> {
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
                Ok((
                    RespValue::BulkString(Some(input.slice(next_start..data_end))),
                    crlf_end,
                ))
            }
            _ => Err(ParseError::LengthError(len)),
        }
    }

    fn parse_checked_array(input: &Bytes, start: usize) -> Result<(RespValue, usize), ParseError> {
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
            // len if len > 0 => {
            len => {
                let mut items = Vec::with_capacity(len as usize);
                for _ in 0..len {
                    let (item, remaining_start_index) = Self::parse_checked_at(input, next_start)?;
                    items.push(item);
                    next_start = remaining_start_index;
                }
                Ok((RespValue::Array(Some(items)), next_start))
            } // len => Err(ParseError::LengthError(len)),
        }
    }

    fn parse_checked_error(input: &Bytes, start: usize) -> Result<(RespValue, usize), ParseError> {
        let (end, next_start) = Self::find_crlf(input, start)?;
        String::from_utf8(input[start..end].into())
            .map(|s| (RespValue::Error(s), next_start))
            .map_err(|_| ParseError::ByteError("Invalid UTF-8 in error message".to_string()))
    }

    fn parse_checked_inline(input: &Bytes, start: usize) -> Result<(RespValue, usize), ParseError> {
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

    fn parse_checked_at(input: &Bytes, start: usize) -> Result<(RespValue, usize), ParseError> {
        if start >= input.len() {
            // Shouldn't be possible but AOF test fails without this and I cba to refactor AOF
            // because it should all be valid if it's being loaded into AOF anyway
            return Err(ParseError::Incomplete);
        }

        match input[start] {
            b'+' => Self::parse_checked_simple_string(input, start + 1),
            b'-' => Self::parse_checked_error(input, start + 1),
            b':' => Self::parse_checked_integer(input, start + 1),
            b'$' => Self::parse_checked_bulk_string(input, start + 1),
            b'*' => Self::parse_checked_array(input, start + 1),
            _ => Self::parse_checked_inline(input, start),
        }
    }

    pub fn parse_checked(input: &Bytes) -> Result<(RespValue, usize), ParseError> {
        Self::parse_checked_at(input, 0)
    }

    fn is_complete(input: &[u8], start: usize) -> Result<usize, ParseError> {
        if start >= input.len() {
            return Err(ParseError::Incomplete);
        }

        let b = input[start];
        match b {
            // wish this was simpler but bulk strings can contain \r\n in the middle of it because
            // fuck you that's why
            b'$' => {
                let length_line_start = start + 1;
                if length_line_start >= input.len() {
                    return Err(ParseError::Incomplete);
                }

                // null bulk string
                if length_line_start + 2 < input.len()
                    && &input[length_line_start..length_line_start + 3] == b"-1\r"
                {
                    match RespValue::find_crlf_u8(input, start) {
                        Err(_) => Err(ParseError::Incomplete),
                        Ok((_, end)) => Ok(end),
                    }
                } else {
                    // get the length
                    let (crlf_start, crlf_end) = RespValue::find_crlf_u8(input, length_line_start)
                        .map_err(|_| ParseError::Incomplete)?;
                    let len: i64 = atoi(&input[length_line_start..crlf_start])
                        .ok_or_else(|| ParseError::ByteError("Invalid bulk length".into()))?;
                    if len < 0 {
                        return Err(ParseError::ByteError("Invalid bulk length".into()));
                    }
                    let len_usize = len as usize; // can't be negative because doesn't start with -

                    let after_data_crlf = crlf_end + len_usize + 2;
                    if after_data_crlf > input.len() {
                        return Err(ParseError::Incomplete);
                    }
                    // we don't need to check bytes at data_end are actually \r\n, faster to go off
                    // given length, surely the length checks stop anything bad happening right...
                    Ok(after_data_crlf)
                }
            }

            // we need to check the array is the right length because buffer could cut it off at
            // the end of an element and we wouldn't know
            b'*' => {
                let length_line_start = start + 1;
                if length_line_start >= input.len() {
                    return Err(ParseError::Incomplete);
                }

                let (crlf_start, crlf_end) = RespValue::find_crlf_u8(input, length_line_start)
                    .map_err(|_| ParseError::Incomplete)?;

                let count: i64 = atoi(&input[length_line_start..crlf_start])
                    .ok_or_else(|| ParseError::ByteError("Invalid array count".into()))?;

                if count < -1 {
                    return Err(ParseError::ByteError("Invalid array count".into()));
                }

                let mut pos = crlf_end;
                for _ in 0..count {
                    pos = RespValue::is_complete(input, pos)?;
                }
                Ok(pos)
            }

            // +,-,:,_,inline
            _ => match RespValue::find_crlf_u8(input, start) {
                Err(_) => Err(ParseError::Incomplete),
                Ok((_, end)) => Ok(end),
            },
        }
    }

    /// ensures the request is complete
    pub fn rough_check(input: &[u8]) -> Result<usize, ParseError> {
        if input.is_empty() {
            return Err(ParseError::Incomplete);
        }

        RespValue::is_complete(input, 0)
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
        let input = &Bytes::from_static(b"+OK\r\n");
        assert!(RespValue::rough_check(input.as_ref()).is_ok());
        let (result, _) = RespValue::parse_checked(input).unwrap();
        assert_eq!(result, RespValue::SimpleString("OK".to_string()));
    }

    #[test]
    fn test_error() {
        let input = &Bytes::from_static(b"-ERR unknown command\r\n");
        assert!(RespValue::rough_check(input.as_ref()).is_ok());
        let (result, _) = RespValue::parse_checked(input).unwrap();
        assert_eq!(result, RespValue::Error("ERR unknown command".to_string()));
    }

    #[test]
    fn test_integer() {
        let input = &Bytes::from_static(b":1000\r\n");
        assert!(RespValue::rough_check(input.as_ref()).is_ok());
        let (result, _) = RespValue::parse_checked(input).unwrap();
        assert_eq!(result, RespValue::Integer(1000));
    }

    #[test]
    fn test_integer_zero() {
        let input = &Bytes::from_static(b":0\r\n");
        assert!(RespValue::rough_check(input.as_ref()).is_ok());
        let (result, _) = RespValue::parse_checked(input).unwrap();
        assert_eq!(result, RespValue::Integer(0));
    }

    #[test]
    fn test_integer_negative() {
        let input = &Bytes::from_static(b":-42\r\n");
        assert!(RespValue::rough_check(input.as_ref()).is_ok());
        let (result, _) = RespValue::parse_checked(input).unwrap();
        assert_eq!(result, RespValue::Integer(-42));
    }

    #[test]
    fn test_bulk_string() {
        let input = &Bytes::from_static(b"$5\r\nhello\r\n");
        assert!(RespValue::rough_check(input.as_ref()).is_ok());
        let (result, _) = RespValue::parse_checked(input).unwrap();
        assert_eq!(
            result,
            RespValue::BulkString(Option::from(Bytes::from_static(b"hello")))
        );
    }

    #[test]
    fn test_bulk_string_empty() {
        let input = &Bytes::from_static(b"$0\r\n\r\n");
        assert!(RespValue::rough_check(input.as_ref()).is_ok());
        let (result, _) = RespValue::parse_checked(input).unwrap();
        assert_eq!(
            result,
            RespValue::BulkString(Option::from(Bytes::from_static(b"")))
        );
    }

    #[test]
    fn test_bulk_string_null() {
        let input = &Bytes::from_static(b"$-1\r\n");
        assert!(RespValue::rough_check(input.as_ref()).is_ok());
        let (result, _) = RespValue::parse_checked(input).unwrap();
        assert_eq!(result, RespValue::BulkString(None));
    }

    #[test]
    fn test_array() {
        let input = &Bytes::from_static(b"*2\r\n$3\r\nfoo\r\n$3\r\nbar\r\n");
        assert!(RespValue::rough_check(input.as_ref()).is_ok());
        let (result, _) = RespValue::parse_checked(input).unwrap();
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
        assert!(RespValue::rough_check(input.as_ref()).is_ok());
        let (result, _) = RespValue::parse_checked(input).unwrap();
        assert_eq!(result, RespValue::Array(vec![].into()));
    }

    #[test]
    fn test_array_null() {
        let input = &Bytes::from_static(b"*-1\r\n");
        assert!(RespValue::rough_check(input.as_ref()).is_ok());
        let (result, _) = RespValue::parse_checked(input).unwrap();
        assert_eq!(result, RespValue::Array(None));
    }

    #[test]
    fn test_array_mixed_types() {
        let input = &Bytes::from_static(b"*3\r\n:1\r\n+OK\r\n$-1\r\n");
        assert!(RespValue::rough_check(input.as_ref()).is_ok());
        let (result, _) = RespValue::parse_checked(input).unwrap();
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
        assert!(RespValue::rough_check(input.as_ref()).is_ok());
        let (result, _) = RespValue::parse_checked(input).unwrap();
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
        assert!(RespValue::parse_checked(input).is_err());
    }

    #[test]
    fn test_incomplete_simple_string() {
        let input = &Bytes::from_static(b"+OK");
        assert!(RespValue::rough_check(input.as_ref()).is_err());
    }

    #[test]
    fn test_invalid_bulk_string_length() {
        let input = &Bytes::from_static(b"$abc\r\nhello\r\n");
        assert!(RespValue::rough_check(input.as_ref()).is_err());
    }

    #[test]
    fn test_negative_bulk_string_length() {
        let input = &Bytes::from_static(b"$-2\r\n");
        assert!(RespValue::rough_check(input.as_ref()).is_err());
    }

    #[test]
    fn test_invalid_array_length() {
        let input = &Bytes::from_static(b"*abc\r\n");
        assert!(RespValue::rough_check(input.as_ref()).is_err());
    }

    #[test]
    fn test_negative_array_length() {
        let input = &Bytes::from_static(b"*-2\r\n");
        assert!(RespValue::rough_check(input.as_ref()).is_err());
    }

    #[test]
    fn test_incomplete_array() {
        let input = &Bytes::from_static(b"*2\r\n:1\r\n");
        assert!(RespValue::rough_check(input.as_ref()).is_err());
    }

    #[test]
    fn test_ok_inline_array() {
        let input = &Bytes::from_static(b"PING\r\n");
        assert!(RespValue::rough_check(input.as_ref()).is_ok());
        let (result, _) = RespValue::parse_checked(input).unwrap();
        assert_eq!(
            result,
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
