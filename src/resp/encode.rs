use super::{
    BulkString, RespArray, RespEncode, RespMap, RespNull, RespNullArray, RespNullBulkString,
    RespSet, SimpleError, SimpleString,
};

/// 为每个枚举实现 encode
const BUF_CAP: usize = 4096;

impl RespEncode for SimpleString {
    fn encode(self) -> Vec<u8> {
        format!("+{}\r\n", self.0).into_bytes()
    }
}

impl RespEncode for SimpleError {
    fn encode(self) -> Vec<u8> {
        format!("-{}\r\n", self.0).into_bytes()
    }
}

/// 给i64类型的添加 encode
impl RespEncode for i64 {
    fn encode(self) -> Vec<u8> {
        // 一般整数，用户不会自己打上+号，而负数用户会主动带上-号，因此 负数时，不用在加一次-号
        let sign = if self < 0 { "" } else { "+" };
        format!(":{}{}\r\n", sign, self).into_bytes()
    }
}

/// $<length>\r\n<data>\r\n
impl RespEncode for Vec<u8> {
    fn encode(self) -> Vec<u8> {
        // format!("${}\r\n{}\r\n", self.len()).into_bytes()
        //    .into_iter()
        //    .chain(self.into_iter())
        //    .collect()
        let mut buf = Vec::with_capacity(self.len() + 16);
        buf.extend_from_slice(&format!("${}\r\n", self.len()).into_bytes());
        buf.extend_from_slice(&self);
        buf.extend_from_slice(b"\r\n");
        buf
    }
}

impl RespEncode for BulkString {
    fn encode(self) -> Vec<u8> {
        self.0.encode()
    }
}

impl RespEncode for RespNullBulkString {
    fn encode(self) -> Vec<u8> {
        // 将固定字符串转换成bytes
        b"$-1\r\n".to_vec()
    }
}
// pattern

impl RespEncode for RespNull {
    fn encode(self) -> Vec<u8> {
        b"_\r\n".to_vec()
    }
}

impl RespEncode for RespNullArray {
    fn encode(self) -> Vec<u8> {
        b"*-1\r\n".to_vec()
    }
}

//*<number-of-elements>\r\n<element-1>...<element-n>
impl RespEncode for RespArray {
    // 需要计算 array的长度。
    fn encode(self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(BUF_CAP);
        buf.extend_from_slice(&format!("*{}\r\n", self.len()).into_bytes());
        for frame in self.0 {
            buf.extend_from_slice(&frame.encode());
        }
        buf
    }
}

//#<t|f>\r\n
impl RespEncode for bool {
    fn encode(self) -> Vec<u8> {
        format!("#{}\r\n", if self { "t" } else { "f" }).into_bytes()
    }
}

//,[<+|->]<integral>[.<fractional>][<E|e>[sign]<exponent>]\r\n
// +e 是rust提供的科学计数法的 format
impl RespEncode for f64 {
    fn encode(self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(BUF_CAP);
        // 大于1亿 就启用科学计数法, 数据过小时，也需要考虑0.000000001234这样的小数位数，这种是使用`1.23456 × 10^(-9)`这种方式计数的
        // 虽然abs去掉的负号，但是小数位数的科学技术仍旧需要保留
        let ret = if self.abs() >= 1e+8 || self.abs() <= 1e-7 {
            format!(",{:+e}\r\n", self)
        } else {
            // 否则，就直接存
            let sign = if self < 0.0 { "" } else { "+" };
            format!(",{}{}\r\n", sign, self)
        };
        buf.extend_from_slice(&ret.into_bytes());
        buf
    }
}

// map数据。
// %<number-of-entries>\r\n<key-1><value-1>...<key-n><value-n>
impl RespEncode for RespMap {
    fn encode(self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(BUF_CAP);
        buf.extend_from_slice(&format!("%{}\r\n", self.len()).into_bytes());
        for (key, value) in self.0 {
            buf.extend_from_slice(&SimpleString::new(key).encode());
            buf.extend_from_slice(&value.encode());
        }
        buf
    }
}

impl RespEncode for RespSet {
    fn encode(self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(BUF_CAP);
        buf.extend_from_slice(&format!("~{}\r\n", self.len()).into_bytes());
        for value in self.0 {
            buf.extend_from_slice(&value.encode());
        }
        buf
    }
}

/// 单元测试
#[cfg(test)]
mod tests {
    use crate::resp::RespFrame;

    use super::*;

    #[test]
    fn test_simple_string_encode() {
        let frame: RespFrame = SimpleString::new("hello").into();
        assert_eq!(frame.encode(), b"+hello\r\n");
    }

    #[test]
    // SimpleError
    fn test_simple_error_encode() {
        let frame: RespFrame = SimpleError::new("error message").into();
        assert_eq!(frame.encode(), b"-error message\r\n");
    }

    // i64
    #[test]
    fn test_simple_integer_encode() {
        let frame: RespFrame = 123.into();
        assert_eq!(frame.encode(), b":+123\r\n");

        let frame: RespFrame = (-123).into();
        assert_eq!(frame.encode(), b":-123\r\n");
    }

    // BulkString
    #[test]
    fn test_bulk_string_encode() {
        let frame: RespFrame = BulkString("hello".as_bytes().to_vec()).into();
        assert_eq!(frame.encode(), b"$5\r\nhello\r\n");
    }

    // RespNullBulkString
    #[test]
    fn test_null_bulk_string_encode() {
        let frame: RespFrame = RespNullBulkString.into();
        assert_eq!(frame.encode(), b"$-1\r\n");
    }

    //RespArray
    // *<number-of-elements>\r\n<element-1>...<element-n>
    #[test]
    fn test_array_encode() {
        // let frame: RespFrame = RespArray::new(vec![
        //     SimpleString::new("set").into(),
        //     SimpleString::new("hello").into(),
        //     SimpleString::new("world").into(),
        // ]).into();
        // assert_eq!(
        //     frame.encode(),
        //     b"*3\r\n+set\r\n+hello\r\n+world\r\n"
        // );
        let frame: RespFrame = RespArray::new(vec![
            BulkString::new("set").into(),
            BulkString::new("hello").into(),
            BulkString::new("world").into(),
        ])
        .into();
        assert_eq!(
            frame.encode(),
            b"*3\r\n$3\r\nset\r\n$5\r\nhello\r\n$5\r\nworld\r\n"
        );
    }

    // RespNullArray
    #[test]
    // RespNullArray
    fn test_null_array_encode() {
        let frame: RespFrame = RespNullArray.into();
        assert_eq!(frame.encode(), b"*-1\r\n");
    }

    //RespNull
    #[test]
    fn test_null_encode() {
        let frame: RespFrame = RespNull.into();
        assert_eq!(frame.encode(), b"_\r\n");
    }

    // bool
    #[test]
    fn test_bool_encode() {
        let frame: RespFrame = true.into();
        assert_eq!(frame.encode(), b"#t\r\n");

        let frame: RespFrame = false.into();
        assert_eq!(frame.encode(), b"#f\r\n");
    }

    //f64
    #[test]
    fn test_double_encode() {
        let frame: RespFrame = 123.456.into();
        assert_eq!(frame.encode(), b",+123.456\r\n");

        let frame: RespFrame = (-123.456).into();
        assert_eq!(frame.encode(), b",-123.456\r\n");

        // 这个有问题
        let frame: RespFrame = 1.23456e+8.into();
        // assert_eq!(String::from_utf8_lossy(&frame.encode()), ",+1.23456e+8\r\n");
        assert_eq!(frame.encode(), b",+1.23456e8\r\n");
        let frame: RespFrame = (-1.23456e-8).into();
        // assert_eq!(String::from_utf8_lossy(&frame.encode()), ",-1.23456e-7\r\n");
        assert_eq!(frame.encode(), b",-1.23456e-8\r\n");
    }

    #[test]
    fn test_map_encode() {
        let mut map = RespMap::new();
        map.insert(
            "hello".to_string(),
            BulkString::new("world".to_string()).into(),
        );
        map.insert("foo".to_string(), (-123456.789).into());

        let frame: RespFrame = map.into();
        assert_eq!(
            frame.encode(),
            // 由于Btreemap内部按照ascii进行了排序，因此，foo的key在hellokey的前面
            b"%2\r\n+foo\r\n,-123456.789\r\n+hello\r\n$5\r\nworld\r\n"
        );
    }

    #[test]
    fn test_set_encode() {
        let frame: RespFrame = RespSet::new([
            RespArray::new([1234.into(), true.into()]).into(),
            BulkString::new("world".to_string()).into(),
        ])
        .into();
        assert_eq!(
            frame.encode(),
            b"~2\r\n*2\r\n:+1234\r\n#t\r\n$5\r\nworld\r\n"
        );
        // assert_eq!(
        //     frame.encode(),
        //     b"~2\r\n:1234\r\n#t\r\n$5\r\nworld\r\n"
        // );
        // assert_eq!(
        //     String::from_utf8_lossy(&frame.encode()),
        //     "~2\r\n:1234\r\n#t\r\n$5\r\nworld\r\n"
        // );
    }
}
