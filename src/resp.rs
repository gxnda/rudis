use atoi::atoi;
use bytes::{Buf, Bytes, BytesMut};
use memchr::memmem;
use thiserror::Error;

#[derive(Debug, PartialEq, Clone)]
pub enum RespValue {
    SimpleString(String),
    BulkString(Option<Bytes>),
    Array(Option<Vec<RespValue>>),
    Integer(i64),
    Error(String),
}

#[derive(Debug, Error)]
pub enum ParseError {
    // Incomplete looks a bit funny, basically it only has stuff in it if an array is incomplete so
    // it can be continued, ((array, incomplete_child_element), should also have support for
    // incomplete nested arrays (that's why it recurses)
    #[error("Incomplete parse")]
    Incomplete(Option<(Vec<RespValue>, Option<Box<ParseError>>)>), // So then Array can persist from incomplete
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

    pub fn is_err(&self) -> bool {
        matches!(self, RespValue::Error(_))
    }

    /// Returns (end, start) around the \r\n, does not include \r\n.
    fn find_crlf(input: &BytesMut, start: usize) -> Result<(usize, usize), ParseError> {
        memmem::find(&input[start..], b"\r\n")
            .map(|i| (start + i, start + i + 2))
            .ok_or(ParseError::Incomplete(None))
    }

    fn parse_simple_string(input: &mut BytesMut, start: usize) -> Result<RespValue, ParseError> {
        let (end, _) = Self::find_crlf(input, start)?;
        // we now know it's valid or malformed, consume the buffer
        input.advance(start);
        let value = String::from_utf8(input.split_to(end - start).into())
            .map_err(|_| ParseError::ByteError("Invalid UTF-8 in simple string".to_string()))?;
        input.advance(2);
        Ok(RespValue::SimpleString(value))
    }

    fn parse_integer(input: &mut BytesMut, start: usize) -> Result<RespValue, ParseError> {
        let (end, next_start) = Self::find_crlf(input, start)?;
        // we now know it's valid or malformed, consume the buffer
        // we don't consume here so then the error message still works
        let int = atoi::<i64>(&input[start..end]).ok_or_else(|| {
            ParseError::NotAnInteger(String::from_utf8_lossy(&input[start..end]).into())
        })?;
        input.advance(next_start); // consume here instead
        Ok(RespValue::Integer(int))
    }

    fn parse_bulk_string(input: &mut BytesMut, start: usize) -> Result<RespValue, ParseError> {
        let (end, next_start) = Self::find_crlf(input, start)?;
        let len = atoi::<i64>(&input[start..end]).ok_or_else(|| {
            ParseError::NotAnInteger(String::from_utf8_lossy(&input[start..end]).into())
        })?;

        match len {
            // null bulk string
            -1 => {
                input.advance(next_start);
                Ok(RespValue::BulkString(None))
            }
            len if len >= 0 => {
                let len = len as usize;
                let data_end = next_start + len;
                let crlf_end = data_end + 2;
                if input.len() < crlf_end || &input[data_end..crlf_end] != b"\r\n" {
                    Err(ParseError::Incomplete(None))
                } else {
                    // is valid, consume buffer
                    input.advance(next_start);
                    let res = input.split_to(len).freeze();
                    input.advance(2);
                    Ok(RespValue::BulkString(Some(res)))
                }
            }
            _ => Err(ParseError::LengthError(len)),
        }
    }

    fn parse_array_from_existing(
        input: &mut BytesMut,
        start: usize,
        mut items: Vec<RespValue>,
    ) -> Result<RespValue, ParseError> {
        input.advance(start);
        for _ in 0..items.capacity().saturating_sub(items.len()) {
            match Self::parse_at(input, 0) {
                Ok(item) => {
                    items.push(item);
                }
                Err(ParseError::Incomplete(Some(inner_items))) => {
                    // contains all valid items up to the incomplete one
                    return Err(ParseError::Incomplete(Some((
                        items,
                        Some(Box::new(ParseError::Incomplete(Some(inner_items)))),
                    ))));
                }
                Err(ParseError::Incomplete(None)) => {
                    return Err(ParseError::Incomplete(Some((items, None))));
                }
                Err(e) => return Err(e),
            }
        }
        Ok(RespValue::Array(Some(items)))
    }

    fn parse_array(input: &mut BytesMut, start: usize) -> Result<RespValue, ParseError> {
        let (end, next_start) = Self::find_crlf(input, start)?;
        let len = atoi::<i64>(&input[start..end]).ok_or_else(|| {
            ParseError::NotAnInteger(String::from_utf8_lossy(&input[start..end]).into())
        })?;
        match len {
            // null
            -1 => Ok(RespValue::Array(None)),
            // empty
            0 => Ok(RespValue::Array(Some(vec![]))),
            // Standard array
            len if len > 0 => {
                let items = Vec::with_capacity(len as usize);
                let res = RespValue::parse_array_from_existing(input, next_start, items);
                res
            }
            len => Err(ParseError::LengthError(len)),
        }
    }

    fn parse_error(input: &mut BytesMut, start: usize) -> Result<RespValue, ParseError> {
        let (end, next_start) = Self::find_crlf(input, start)?;
        let str = String::from_utf8(input[start..end].into())
            .map_err(|_| ParseError::ByteError("Invalid UTF-8 in error message".to_string()))?;
        input.advance(next_start);
        Ok(RespValue::Error(str))
    }

    fn parse_inline(input: &mut BytesMut, start: usize) -> Result<RespValue, ParseError> {
        let (end, _) = Self::find_crlf(input, start)?;
        let s = &input[start..end];
        if s.contains(&b' ') {
            return Err(ParseError::ByteError(
                format!(
                    "Inline command contains a space: {}",
                    str::from_utf8(&s).unwrap_or("Error parsing command")
                )
                .to_string(),
            ));
        }

        // valid, start consuming
        input.advance(start);
        let res = RespValue::Array(
            vec![RespValue::BulkString(Some(
                input.split_to(end - start).freeze(),
            ))]
            .into(),
        );
        input.advance(2);
        Ok(res)
    }

    fn parse_at(input: &mut BytesMut, start: usize) -> Result<RespValue, ParseError> {
        if start >= input.len() {
            // no current items in the array, no child items that may be incomplete
            return Err(ParseError::Incomplete(None));
        }

        match input[start] {
            // uses index because we don't want to consume type definition if invalid
            b'+' => Self::parse_simple_string(input, start + 1),
            b'-' => Self::parse_error(input, start + 1),
            b':' => Self::parse_integer(input, start + 1),
            b'$' => Self::parse_bulk_string(input, start + 1),
            b'*' => Self::parse_array(input, start + 1),
            _ => Self::parse_inline(input, start),
        }
    }

    /// Takes in &mut Bytesmut, uses split_to to take Bytes when valid
    pub fn parse(input: &mut BytesMut) -> Result<RespValue, ParseError> {
        Self::parse_at(input, 0)
    }

    /// Attempts to parse, continuing on from the last that was incomplete
    pub fn parse_from_incomplete(
        input: &mut BytesMut,
        incomplete: ParseError,
    ) -> Result<RespValue, ParseError> {
        match incomplete {
            ParseError::Incomplete(Some((mut items, Some(rest)))) => {
                // rest is always a valid addition to items, since it would be in the
                // parse_array 1..len loop
                match RespValue::parse_from_incomplete(input, *rest) {
                    Ok(item) => items.push(item),
                    // Tried to parse nested section, but we still don't have enough to complete it
                    Err(ParseError::Incomplete(inner)) => {
                        return Err(ParseError::Incomplete(Some((
                            items,
                            Some(Box::new(ParseError::Incomplete(inner))),
                        ))))
                    }
                    Err(e) => return Err(e),
                }
                let start = if input.starts_with(b"\r\n") { 2 } else { 0 };
                return RespValue::parse_array_from_existing(input, start, items);
            }
            ParseError::Incomplete(Some((items, None))) => {
                let start = if input.starts_with(b"\r\n") { 2 } else { 0 };
                return RespValue::parse_array_from_existing(input, start, items);
            }
            _ => panic!("Only ParseError::Incomplete should be passed into parse_from_incomplete"),
        }
    }

    /// Don't use!! only for testing
    pub fn parse_bytes(bytes: &Bytes) -> Result<RespValue, ParseError> {
        RespValue::parse(&mut BytesMut::from(bytes.as_ref()))
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
        bytes.extend(e.as_bytes());
        bytes.extend(b"\r\n");
        bytes
    }

    fn serialize_integer(i: i64) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.push(b':');
        bytes.extend(i.to_string().as_bytes()); // normally small, itoa slower overall in
                                                // redis-benchmark, so this will do, I could add
                                                // some sort of threshold where it switches
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
        let result = RespValue::parse_bytes(input).unwrap();
        assert_eq!(result, RespValue::SimpleString("OK".to_string()));
    }

    #[test]
    fn test_error() {
        let input = &Bytes::from_static(b"-ERR unknown command\r\n");
        let result = RespValue::parse_bytes(input).unwrap();
        assert_eq!(result, RespValue::Error("ERR unknown command".to_string()));
    }

    #[test]
    fn test_integer() {
        let input = &Bytes::from_static(b":1000\r\n");
        let result = RespValue::parse_bytes(input).unwrap();
        assert_eq!(result, RespValue::Integer(1000));
    }

    #[test]
    fn test_integer_zero() {
        let input = &Bytes::from_static(b":0\r\n");
        let result = RespValue::parse_bytes(input).unwrap();
        assert_eq!(result, RespValue::Integer(0));
    }

    #[test]
    fn test_integer_negative() {
        let input = &Bytes::from_static(b":-42\r\n");
        let result = RespValue::parse_bytes(input).unwrap();
        assert_eq!(result, RespValue::Integer(-42));
    }

    #[test]
    fn test_bulk_string() {
        let input = &Bytes::from_static(b"$5\r\nhello\r\n");
        let result = RespValue::parse_bytes(input).unwrap();
        assert_eq!(
            result,
            RespValue::BulkString(Option::from(Bytes::from_static(b"hello")))
        );
    }

    #[test]
    fn test_bulk_string_empty() {
        let input = &Bytes::from_static(b"$0\r\n\r\n");
        let result = RespValue::parse_bytes(input).unwrap();
        assert_eq!(
            result,
            RespValue::BulkString(Option::from(Bytes::from_static(b"")))
        );
    }

    #[test]
    fn test_bulk_string_null() {
        let input = &Bytes::from_static(b"$-1\r\n");
        let result = RespValue::parse_bytes(input).unwrap();
        assert_eq!(result, RespValue::BulkString(None));
    }

    #[test]
    fn test_array() {
        // assertion `left == right` failed
        //   left: Array(Some([BulkString(Some(b"foo")), Array(Some([BulkString(Some(b"bar"))]))]))
        //  right: Array(Some([BulkString(Some(b"foo")), BulkString(Some(b"bar"))]))
        let input = &Bytes::from_static(b"*2\r\n$3\r\nfoo\r\n$3\r\nbar\r\n");
        let result = RespValue::parse_bytes(input).unwrap();
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
        let result = RespValue::parse_bytes(input).unwrap();
        assert_eq!(result, RespValue::Array(vec![].into()));
    }

    #[test]
    fn test_array_null() {
        let input = &Bytes::from_static(b"*-1\r\n");
        let result = RespValue::parse_bytes(input).unwrap();
        assert_eq!(result, RespValue::Array(None));
    }

    #[test]
    fn test_array_mixed_types() {
        let input = &Bytes::from_static(b"*3\r\n:1\r\n+OK\r\n$-1\r\n");
        let result = RespValue::parse_bytes(input).unwrap();
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
        let result = RespValue::parse_bytes(input).unwrap();
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
        let result = RespValue::parse_bytes(input);
        assert!(result.is_err());
    }

    #[test]
    fn test_incomplete_simple_string() {
        let input = &Bytes::from_static(b"+OK");
        let result = RespValue::parse_bytes(input);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_bulk_string_length() {
        let input = &Bytes::from_static(b"$abc\r\nhello\r\n");
        let result = RespValue::parse_bytes(input);
        assert!(result.is_err());
    }

    #[test]
    fn test_negative_bulk_string_length() {
        let input = &Bytes::from_static(b"$-2\r\n");
        let result = RespValue::parse_bytes(input);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_array_length() {
        let input = &Bytes::from_static(b"*abc\r\n");
        let result = RespValue::parse_bytes(input);
        assert!(result.is_err());
    }

    #[test]
    fn test_negative_array_length() {
        let input = &Bytes::from_static(b"*-2\r\n");
        let result = RespValue::parse_bytes(input);
        assert!(result.is_err());
    }

    #[test]
    fn test_incomplete_array() {
        let input = &Bytes::from_static(b"*2\r\n:1\r\n");
        let result = RespValue::parse_bytes(input);
        assert!(result.is_err());
    }

    #[test]
    fn test_ok_inline_array() {
        let input = &Bytes::from_static(b"PING\r\n");
        let result = RespValue::parse_bytes(input);
        assert!(result.is_ok());
        let inline_ping = result.unwrap();
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
