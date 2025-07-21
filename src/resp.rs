use std::slice::Iter;

#[derive(Debug, PartialEq)]
pub enum RespValue {
    SimpleString(String),
    BulkString(Option<String>),  // Use Option for bulk strings
    Array(Option<Vec<RespValue>>),  // Use Option for arrays
    Integer(i64),
    Error(String),
}

impl RespValue {
    fn parse_str(chars: &mut Iter<u8>) -> Result<String, &'static str> {
        let mut bytes = Vec::new();
        while let Some(&byte) = chars.next() {
            if byte == b'\r' {
                if chars.next() != Some(&b'\n') {
                    return Err("Expected LF after CR");
                }
                return String::from_utf8(bytes).map_err(|_| "Invalid UTF-8");
            }
            bytes.push(byte);
        }
        Err("Unexpected end of input")
    }

    fn parse_int(chars: &mut Iter<u8>) -> Result<i64, &'static str> {
        let s = Self::parse_str(chars)?;
        s.parse().map_err(|_| "Invalid integer")
    }

    fn parse_bulk_string(chars: &mut Iter<u8>) -> Result<Option<String>, &'static str> {
        let len = match Self::parse_int(chars)? {
            -1 => return Ok(None),  // Null bulk string
            len if len >= 0 => len as usize,
            _ => return Err("Invalid bulk string length"),
        };
        let mut bytes = Vec::with_capacity(len);
        for _ in 0..len {
            bytes.push(*chars.next().ok_or("Unexpected end of bulk string")?);
        }

        if chars.next() != Some(&b'\r') || chars.next() != Some(&b'\n') {
            return Err("Bulk string not terminated with CRLF");
        }

        String::from_utf8(bytes)
            .map(Some)
            .map_err(|_| "Invalid UTF-8 in bulk string")
    }

    fn parse_array(chars: &mut Iter<u8>) -> Result<Option<Vec<RespValue>>, &'static str> {
        let len = match Self::parse_int(chars)? {
            -1 => return Ok(None),  // Null array
            len if len >= 0 => len as usize,
            _ => return Err("Invalid array length"),
        };

        let mut array = Vec::with_capacity(len);
        for _ in 0..len {
            array.push(Self::parse_iter(chars)?);
        }
        Ok(Some(array))
    }

    pub fn parse(input: &[u8]) -> Result<RespValue, &'static str> {
        let mut iter = input.iter();
        Self::parse_iter(&mut iter)
    }

    fn parse_iter(chars: &mut Iter<u8>) -> Result<RespValue, &'static str> {
        match chars.next().ok_or("Unexpected end of input")? {
            b'+' => Ok(RespValue::SimpleString(Self::parse_str(chars)?)),
            b'-' => Ok(RespValue::Error(Self::parse_str(chars)?)),
            b':' => Ok(RespValue::Integer(Self::parse_int(chars)?)),
            b'$' => Ok(RespValue::BulkString(Self::parse_bulk_string(chars)?)),
            b'*' => Ok(RespValue::Array(Self::parse_array(chars)?)),
            _ => Err("Invalid first byte"),
        }
    }

    pub fn serialize(&self) -> Vec<u8> {
        match self {
            RespValue::SimpleString(s) => Self::serialize_simple_string(s),
            RespValue::BulkString(opt) => Self::serialize_bulk_string(opt.as_deref()),
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

    fn serialize_bulk_string(s: Option<&str>) -> Vec<u8> {
        match s {
            Some(s) => {
                let mut bytes = Vec::new();
                bytes.push(b'$');
                bytes.extend(s.len().to_string().as_bytes());
                bytes.extend(b"\r\n");
                bytes.extend(s.as_bytes());
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
        let result = RespValue::parse(input).unwrap();
        assert_eq!(result, RespValue::SimpleString("OK".to_string()));
    }

    #[test]
    fn test_error() {
        let input = b"-ERR unknown command\r\n";
        let result = RespValue::parse(input).unwrap();
        assert_eq!(result, RespValue::Error("ERR unknown command".to_string()));
    }

    #[test]
    fn test_integer() {
        let input = b":1000\r\n";
        let result = RespValue::parse(input).unwrap();
        assert_eq!(result, RespValue::Integer(1000));
    }

    #[test]
    fn test_integer_zero() {
        let input = b":0\r\n";
        let result = RespValue::parse(input).unwrap();
        assert_eq!(result, RespValue::Integer(0));
    }

    #[test]
    fn test_integer_negative() {
        let input = b":-42\r\n";
        let result = RespValue::parse(input).unwrap();
        assert_eq!(result, RespValue::Integer(-42));
    }

    #[test]
    fn test_bulk_string() {
        let input = b"$5\r\nhello\r\n";
        let result = RespValue::parse(input).unwrap();
        assert_eq!(result, RespValue::BulkString(Option::from("hello".to_string())));
    }

    #[test]
    fn test_bulk_string_empty() {
        let input = b"$0\r\n\r\n";
        let result = RespValue::parse(input).unwrap();
        assert_eq!(result, RespValue::BulkString(Option::from("".to_string())));
    }

    #[test]
    fn test_bulk_string_null() {
        let input = b"$-1\r\n";
        let result = RespValue::parse(input).unwrap();
        assert_eq!(result, RespValue::BulkString(None));
    }

    #[test]
    fn test_array() {
        let input = b"*2\r\n$3\r\nfoo\r\n$3\r\nbar\r\n";
        let result = RespValue::parse(input).unwrap();
        assert_eq!(
            result,
            RespValue::Array(Option::from(vec![
                RespValue::BulkString(Option::from("foo".to_string())),
                RespValue::BulkString(Option::from("bar".to_string()))
            ]))
        );
    }

    #[test]
    fn test_array_empty() {
        let input = b"*0\r\n";
        let result = RespValue::parse(input).unwrap();
        assert_eq!(result, RespValue::Array(vec![].into()));
    }

    #[test]
    fn test_array_null() {
        let input = b"*-1\r\n";
        let result = RespValue::parse(input).unwrap();
        assert_eq!(result, RespValue::Array(None));
    }

    #[test]
    fn test_array_mixed_types() {
        let input = b"*3\r\n:1\r\n+OK\r\n$-1\r\n";
        let result = RespValue::parse(input).unwrap();
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
        let result = RespValue::parse(input).unwrap();
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
}