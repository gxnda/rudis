use atoi::atoi;
use bytes::{Buf, BufMut, Bytes, BytesMut};
use memchr::memmem;
use thiserror::Error;

#[derive(Debug, PartialEq, Clone)]
pub enum RespValue {
    SimpleString(Bytes),
    BulkString(Option<Bytes>),
    Array(Option<Vec<RespValue>>),
    Integer(i64),
    Error(Bytes),
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

#[inline]
pub fn digits(num: i64) -> usize {
    // Match statement will probably always be faster due to the distribution of RESP lengths
    match num {
        ..=-1 => 2, // negative sign, the only valid negative resp length is -1
        0..=9 => 1,
        10..=99 => 2,
        100..=999 => 3,
        1000..=9999 => 4,
        _ => num.ilog10() as usize + 1,
    }
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
    fn find_crlf(input: &BytesMut) -> Result<(usize, usize), ParseError> {
        memmem::find(input, b"\r\n")
            .map(|i| (i, i + 2))
            .ok_or(ParseError::Incomplete(None))
    }

    fn parse_simple_string(input: &mut BytesMut) -> Result<RespValue, ParseError> {
        let (end, _) = Self::find_crlf(input)?;
        // we now know it's valid or malformed, consume the buffer
        input.advance(1); // get rid of prefix;
        let str = input.split_to(end - 1).freeze();
        input.advance(2);
        Ok(RespValue::SimpleString(str))
    }

    fn parse_integer(input: &mut BytesMut) -> Result<RespValue, ParseError> {
        let (end, next_start) = Self::find_crlf(input)?;
        // we now know it's valid or malformed, consume the buffer
        // we don't consume here so then the error message still works
        let int = atoi::<i64>(&input[1..end]).ok_or_else(|| {
            ParseError::NotAnInteger(String::from_utf8_lossy(&input[1..end]).into())
        })?;
        input.advance(next_start); // consume here instead
        Ok(RespValue::Integer(int))
    }

    fn parse_bulk_string(input: &mut BytesMut) -> Result<RespValue, ParseError> {
        let (end, next_start) = Self::find_crlf(input)?;
        let len = atoi::<i64>(&input[1..end]).ok_or_else(|| {
            ParseError::NotAnInteger(String::from_utf8_lossy(&input[1..end]).into())
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
        mut items: Vec<RespValue>,
    ) -> Result<RespValue, ParseError> {
        for _ in 0..items.capacity().saturating_sub(items.len()) {
            match Self::parse(input) {
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

    fn parse_array(input: &mut BytesMut) -> Result<RespValue, ParseError> {
        let (end, first_element_start) = Self::find_crlf(input)?;
        let len = atoi::<i64>(&input[1..end]).ok_or_else(|| {
            ParseError::NotAnInteger(String::from_utf8_lossy(&input[1..end]).into())
        })?;
        input.advance(first_element_start);
        match len {
            // null
            -1 => Ok(RespValue::Array(None)),
            // empty
            0 => Ok(RespValue::Array(Some(vec![]))),
            // Standard array
            len if len > 0 => {
                let items = Vec::with_capacity(len as usize);

                RespValue::parse_array_from_existing(input, items)
            }
            len => Err(ParseError::LengthError(len)),
        }
    }

    fn parse_error(input: &mut BytesMut) -> Result<RespValue, ParseError> {
        let (end, _) = Self::find_crlf(input)?;
        input.advance(1); // get rid of prefix;
        let err = input.split_to(end - 1).freeze();
        input.advance(2);
        Ok(RespValue::Error(err))
    }

    fn parse_inline(input: &mut BytesMut) -> Result<RespValue, ParseError> {
        let (end, _) = Self::find_crlf(input)?;
        let s = &input[0..end];
        if s.contains(&b' ') {
            return Err(ParseError::ByteError(
                format!(
                    "Inline command contains a space: {}",
                    str::from_utf8(s).unwrap_or("Error parsing command")
                )
                .to_string(),
            ));
        }

        // valid, start consuming
        let res = RespValue::Array(
            vec![RespValue::BulkString(Some(input.split_to(end).freeze()))].into(),
        );
        input.advance(2);
        Ok(res)
    }

    /// Takes in &mut Bytesmut, uses split_to to take Bytes when valid,
    /// if it's not valid, parse error will be returned.
    ///
    /// If it could hypothetically be valid, it will be returned within ParseError::Incomplete
    /// This is a recursive type to be able to parse incomplete nested arrays.
    pub fn parse(input: &mut BytesMut) -> Result<RespValue, ParseError> {
        if input.is_empty() {
            // no current items in the array, no child items that may be incomplete
            return Err(ParseError::Incomplete(None));
        }

        match input[0] {
            // uses index because we don't want to consume type definition if invalid
            b'+' => Self::parse_simple_string(input),
            b'-' => Self::parse_error(input),
            b':' => Self::parse_integer(input),
            b'$' => Self::parse_bulk_string(input),
            b'*' => Self::parse_array(input),
            _ => Self::parse_inline(input),
        }
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
                RespValue::parse_array_from_existing(input, items)
            }
            ParseError::Incomplete(Some((items, None))) => {
                RespValue::parse_array_from_existing(input, items)
            }
            _ => panic!("Only ParseError::Incomplete should be passed into parse_from_incomplete"),
        }
    }

    /// Don't use!! only for testing
    pub fn parse_bytes(bytes: &Bytes) -> Result<RespValue, ParseError> {
        RespValue::parse(&mut BytesMut::from(bytes.as_ref()))
    }

    fn serialized_len(&self) -> usize {
        match self {
            RespValue::SimpleString(s) => 1 + s.len() + 2,
            RespValue::Error(e) => 1 + e.len() + 2,
            RespValue::BulkString(Some(s)) => {
                1 + digits(s.len().try_into().expect("Can't have negative len")) + 2 + s.len() + 2
            }
            RespValue::BulkString(None) => {
                b"$-1\r\n".len() // should optimise in compiler
            }
            RespValue::Array(Some(arr)) => arr.iter().map(|el| el.serialized_len()).sum(),
            RespValue::Array(None) => {
                b"*-1\r\n".len() // should optimise in compiler
            }
            RespValue::Integer(i) => 1 + digits(*i) + 2,
        }
    }

    pub fn serialize(self) -> Bytes {
        // pre alloc return bytes
        let len = self.serialized_len();
        match self {
            RespValue::SimpleString(s) => Self::serialize_simple_string(s, len),
            RespValue::BulkString(opt) => Self::serialize_bulk_string(opt, len),
            RespValue::Array(opt) => Self::serialize_array(opt, len),
            RespValue::Integer(i) => Self::serialize_integer(i, len),
            RespValue::Error(e) => Self::serialize_error(e, len),
        }
    }

    fn serialize_simple_string(s: Bytes, len: usize) -> Bytes {
        let mut bytes = BytesMut::with_capacity(len);
        bytes.put_u8(b'+');
        bytes.extend(s);
        bytes.extend(b"\r\n");
        bytes.freeze()
    }

    fn serialize_error(e: Bytes, len: usize) -> Bytes {
        let mut bytes = BytesMut::with_capacity(len);
        bytes.put_u8(b'-');
        bytes.extend(e);
        bytes.extend(b"\r\n");
        bytes.freeze()
    }

    fn serialize_integer(i: i64, len: usize) -> Bytes {
        let mut bytes = BytesMut::with_capacity(len);
        bytes.put_u8(b':');
        bytes.extend(i.to_string().as_bytes()); // normally small, itoa slower overall in
                                                // redis-benchmark, so this will do, I could add
                                                // some sort of threshold where it switches
        bytes.extend(b"\r\n");
        bytes.freeze()
    }

    fn serialize_bulk_string(s: Option<Bytes>, len: usize) -> Bytes {
        match s {
            Some(s) => {
                let mut bytes = BytesMut::with_capacity(len);
                bytes.put_u8(b'$');
                bytes.extend(s.len().to_string().as_bytes());
                bytes.put_slice(b"\r\n");
                bytes.extend(s);
                bytes.put_slice(b"\r\n");
                bytes.freeze()
            }
            None => Bytes::from_static(b"$-1\r\n"),
        }
    }

    fn serialize_array(opt: Option<Vec<RespValue>>, len: usize) -> Bytes {
        match opt {
            Some(elements) => {
                let mut bytes = BytesMut::with_capacity(len);
                bytes.put_u8(b'*');
                bytes.extend(Bytes::from(elements.len().to_string()));
                bytes.extend(b"\r\n");
                for elem in elements {
                    bytes.extend(elem.serialize());
                }
                bytes.freeze()
            }
            None => Bytes::from_static(b"*-1\r\n"),
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
        assert_eq!(result, RespValue::SimpleString(Bytes::from_static(b"OK")));
    }

    #[test]
    fn test_error() {
        let input = &Bytes::from_static(b"-ERR unknown command\r\n");
        let result = RespValue::parse_bytes(input).unwrap();
        assert_eq!(
            result,
            RespValue::Error(Bytes::from_static(b"ERR unknown command"))
        );
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
                RespValue::SimpleString(Bytes::from_static(b"OK")),
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
                RespValue::SimpleString(Bytes::from_static(b"OK"))
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
        let value = RespValue::SimpleString(Bytes::from_static(b"OK"));
        assert_eq!(value.serialize().as_ref(), b"+OK\r\n");
    }

    #[test]
    fn test_serialize_error() {
        let value = RespValue::Error(Bytes::from_static(b"ERR unknown command"));
        assert_eq!(value.serialize().as_ref(), b"-ERR unknown command\r\n");
    }

    #[test]
    fn test_serialize_integer_positive() {
        let value = RespValue::Integer(1000);
        assert_eq!(value.serialize().as_ref(), b":1000\r\n");
    }

    #[test]
    fn test_serialize_integer_zero() {
        let value = RespValue::Integer(0);
        assert_eq!(value.serialize().as_ref(), b":0\r\n");
    }

    #[test]
    fn test_serialize_integer_negative() {
        let value = RespValue::Integer(-42);
        assert_eq!(value.serialize().as_ref(), b":-42\r\n");
    }

    #[test]
    fn test_serialize_bulk_string() {
        let value = RespValue::BulkString(Some(Bytes::from_static(b"hello")));
        assert_eq!(value.serialize().as_ref(), b"$5\r\nhello\r\n");
    }

    #[test]
    fn test_serialize_bulk_string_empty() {
        let value = RespValue::BulkString(Some(Bytes::from_static(b"")));
        assert_eq!(value.serialize().as_ref(), b"$0\r\n\r\n");
    }

    #[test]
    fn test_serialize_bulk_string_null() {
        let value = RespValue::BulkString(None);
        assert_eq!(value.serialize().as_ref(), b"$-1\r\n");
    }

    #[test]
    fn test_serialize_array() {
        let value = RespValue::Array(Some(vec![
            RespValue::BulkString(Some(Bytes::from_static(b"foo"))),
            RespValue::BulkString(Some(Bytes::from_static(b"bar"))),
        ]));
        assert_eq!(
            value.serialize().as_ref(),
            b"*2\r\n$3\r\nfoo\r\n$3\r\nbar\r\n"
        );
    }

    #[test]
    fn test_serialize_array_empty() {
        let value = RespValue::Array(Some(vec![]));
        assert_eq!(value.serialize().as_ref(), b"*0\r\n");
    }

    #[test]
    fn test_serialize_array_null() {
        let value = RespValue::Array(None);
        assert_eq!(value.serialize().as_ref(), b"*-1\r\n");
    }

    #[test]
    fn test_serialize_array_mixed_types() {
        let value = RespValue::Array(Some(vec![
            RespValue::Integer(1),
            RespValue::SimpleString(Bytes::from_static(b"OK")),
            RespValue::BulkString(None),
        ]));
        assert_eq!(value.serialize().as_ref(), b"*3\r\n:1\r\n+OK\r\n$-1\r\n");
    }

    #[test]
    fn test_serialize_nested_array() {
        let value = RespValue::Array(Some(vec![
            RespValue::Array(Some(vec![RespValue::Integer(1)])),
            RespValue::SimpleString(Bytes::from_static(b"OK")),
        ]));
        assert_eq!(value.serialize().as_ref(), b"*2\r\n*1\r\n:1\r\n+OK\r\n");
    }
}
