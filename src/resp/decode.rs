use crate::resp::{RespDecode, RespDecodeError, RespFrame, SimpleString};
use bytes::{Buf, BytesMut};

use super::{
    BulkString, RespArray, RespMap, RespNull, RespNullArray, RespNullBulkString, RespSet,
    SimpleError,
};

// impl RespDecode for BytesMut {
//     fn decode(buf: Self) -> Result<RespFrame, RespDecodeError> {
//         todo!()
//     }
// }
const CRLF: &[u8] = b"\r\n";
const CRLF_LEN: usize = CRLF.len();

impl RespDecode for RespFrame {
    const PREFIX: &'static str = "";
    fn decode(buf: &mut BytesMut) -> Result<Self, RespDecodeError> {
        let a = String::from_utf8_lossy(buf);
        println!("测试结果={:?}", a);
        let mut iter = buf.iter().peekable();
        match iter.peek() {
            Some(b'+') => {
                let frame = SimpleString::decode(buf)?;
                Ok(frame.into())
            }
            Some(b'-') => {
                let frame = SimpleError::decode(buf)?;
                Ok(frame.into())
            }
            Some(b':') => {
                let frame = i64::decode(buf)?;
                Ok(frame.into())
            }
            Some(b'$') => {
                // try null null bulk strings
                match RespNullBulkString::decode(buf) {
                    Ok(frame) => Ok(frame.into()),
                    Err(RespDecodeError::NotComplete) => Err(RespDecodeError::NotComplete),
                    // 其他情况时，采用 bulks Strings
                    Err(_) => {
                        let frame = BulkString::decode(buf)?;
                        Ok(frame.into())
                    }
                }
            }
            Some(b'*') => {
                // 数组 需要防止给一个空数组 try null array first
                match RespNullArray::decode(buf) {
                    // 如果是个空，直接返回
                    Ok(frame) => Ok(frame.into()),
                    // 如果是未接收完成的错误，直接返回
                    Err(RespDecodeError::NotComplete) => Err(RespDecodeError::NotComplete),
                    // 其他情况时，按照Array的方式来处理
                    Err(_) => {
                        let frame = RespArray::decode(buf)?;
                        Ok(frame.into())
                    }
                }
            }
            Some(b'_') => {
                let frame = RespNull::decode(buf)?;
                Ok(frame.into())
            }
            Some(b'#') => {
                let frame = bool::decode(buf)?;
                Ok(frame.into())
            }
            Some(b'%') => {
                let frame = RespMap::decode(buf)?;
                Ok(frame.into())
            }
            Some(b'~') => {
                let frame = RespSet::decode(buf)?;
                Ok(frame.into())
            }
            _ => Err(RespDecodeError::InvalidFrameType(format!(
                "unknown frame type: {:?}",
                buf
            ))),
        }
    }

    // 获取预期的长度
    fn expect_length(buf: &[u8]) -> Result<usize, RespDecodeError> {
        let mut iter = buf.iter().peekable();
        match iter.peek() {
            Some(b'*') => RespArray::expect_length(buf),
            // ~
            Some(b'~') => RespSet::expect_length(buf),
            Some(b'%') => RespMap::expect_length(buf),
            Some(b'$') => BulkString::expect_length(buf),
            Some(b':') => i64::expect_length(buf),
            Some(b'+') => SimpleString::expect_length(buf),
            Some(b'-') => SimpleError::expect_length(buf),
            Some(b'#') => bool::expect_length(buf),
            Some(b',') => f64::expect_length(buf),
            Some(b'_') => RespNull::expect_length(buf),
            // 当开头不满足以上分支时，表示不在预期处理内，或者没接收完成
            _ => Err(RespDecodeError::NotComplete),
        }
    }
}

/// 解码SimpleString
impl RespDecode for SimpleString {
    const PREFIX: &'static str = "+";
    fn decode(buf: &mut BytesMut) -> Result<Self, RespDecodeError> {
        /*        if buf.len() < 3 {
            return Err(RespDecodeError::NotComplete);
        }
        if !buf.starts_with(b"+") {
            return Err(RespDecodeError::InvalidFrameType(format!("except: SimpleString(+), got:{:?}", buf)));
        } */
        // simple String +OK\r\n
        // search for "\r\n"
        /*         let mut end = 0;
        for i in 0..buf.len() -1 {
            if buf[i] == b'\r' && buf[i+1] == b'\n' {
                // 获取到\r的下标
                end = i;
                break;
            }
        } */
        // 判定end不为0，否则抛出异常
        /*         if end == 0 {
            return Err(RespDecodeError::NotComplete);
        } */
        // 解码得到SimpleString
        // get String is OK
        // split_to  contains elements [0, at)
        /* let data = buf.split_to(end+2);
        let s = String::from_utf8_lossy(&data[1..end]);
        Ok(SimpleString::new(s.to_string())) */

        // 这里可以采用统一封装好的方法来进行计算
        let end = extract_simple_frame_data(buf, Self::PREFIX)?;
        let data = buf.split_to(end + 2);
        let s = String::from_utf8_lossy(&data[1..end]);
        Ok(SimpleString::new(s.to_string()))
    }

    // e.g +OK\r\n
    fn expect_length(buf: &[u8]) -> Result<usize, RespDecodeError> {
        // e.g +OK\r\n end = 3 + CRLF_LEN(2) = 5; 因此，整个帧
        let end = extract_simple_frame_data(buf, Self::PREFIX)?;
        Ok(end + CRLF_LEN)
    }
}

impl RespDecode for SimpleError {
    const PREFIX: &'static str = "-";
    fn decode(buf: &mut BytesMut) -> Result<Self, RespDecodeError> {
        let end = extract_simple_frame_data(buf, Self::PREFIX)?;
        let data = buf.split_to(end + 2);
        let s = String::from_utf8_lossy(&data[1..end]);
        Ok(SimpleError::new(s.to_string()))
    }

    // e.g -Error message\r\n
    fn expect_length(buf: &[u8]) -> Result<usize, RespDecodeError> {
        let end = extract_simple_frame_data(buf, Self::PREFIX)?;
        Ok(end + CRLF_LEN)
    }
}

impl RespDecode for RespNull {
    const PREFIX: &'static str = "_\r\n";

    fn decode(buf: &mut BytesMut) -> Result<Self, RespDecodeError> {
        extract_fixed_data(buf, Self::PREFIX, "Null")?;
        Ok(RespNull)
    }

    fn expect_length(_buf: &[u8]) -> Result<usize, RespDecodeError> {
        Ok(3)
    }
}

impl RespDecode for RespNullArray {
    const PREFIX: &'static str = "*-1\r\n";
    fn decode(buf: &mut BytesMut) -> Result<Self, RespDecodeError> {
        // 这种空的东西，只需要判定格式是否正确即可
        extract_fixed_data(buf, Self::PREFIX, "RespNullArray")?;
        Ok(RespNullArray)
    }

    fn expect_length(_buf: &[u8]) -> Result<usize, RespDecodeError> {
        Ok(4)
    }
}

impl RespDecode for RespNullBulkString {
    const PREFIX: &'static str = "$-1\r\n";
    fn decode(buf: &mut BytesMut) -> Result<Self, RespDecodeError> {
        extract_fixed_data(buf, Self::PREFIX, "RespNullBulkString")?;
        Ok(RespNullBulkString)
    }

    fn expect_length(_buf: &[u8]) -> Result<usize, RespDecodeError> {
        Ok(5)
    }
}

// e.g :-456\r\n
impl RespDecode for i64 {
    const PREFIX: &'static str = ":";
    fn decode(buf: &mut BytesMut) -> Result<Self, RespDecodeError> {
        let end = extract_simple_frame_data(buf, Self::PREFIX)?;
        let data = buf.split_to(end + 2);
        let s = String::from_utf8_lossy(&data[1..end]);
        Ok(s.parse()?)
    }

    fn expect_length(buf: &[u8]) -> Result<usize, RespDecodeError> {
        let end = extract_simple_frame_data(buf, Self::PREFIX)?;
        Ok(end + CRLF_LEN)
    }
}
// / 数据格式
// $<length>\r\n<data>\r\n
impl RespDecode for BulkString {
    const PREFIX: &'static str = "$";
    fn decode(buf: &mut BytesMut) -> Result<Self, RespDecodeError> {
        let (end, len) = parse_length(buf, Self::PREFIX)?;
        println!("预测得到的结果，end={}, len={}", end, len);
        // end =2,
        // 获取从标识符$之后的\r\n开始真是的内容，但是包含了\r\n的结尾
        let remained = &buf[end + CRLF_LEN..];
        // 感知 上述结果，是否比 $\r\n{n}\r\n小，如果是，则表示出现异常了
        if remained.len() < len + CRLF_LEN {
            return Err(RespDecodeError::NotComplete);
        }
        // 遗弃从\r开始 也就是$5之后的\r\n
        buf.advance(end + CRLF_LEN);
        // 这里之所以加上 CRLF_LEN有两方面的考虑
        // 第一，在某些协议中，如果RESP，数据后面会跟随 \r\n 作为结束标识，因此需要将这部分提取出来
        // 第二，如果我们将 \r\n 部分遗留在buf中，那么就会对下一次的输入产生影响，这里\r\n就成了下一次输入的脏数据了
        let data = buf.split_to(len + CRLF_LEN);
        Ok(BulkString::new(data[..len].to_vec()))

        // let end = extract_simple_frame_data(buf, "\r\n")?;
        // let data = buf.split_to(end + 2);
        // let s = String::from_utf8_lossy(&data[1..end]);
        // let len = s.parse()?;
        // Ok(BulkString::new(data[..len].to_vec()))
    }

    // e.g $5\r\nhello\r\n
    fn expect_length(buf: &[u8]) -> Result<usize, RespDecodeError> {
        let (end, len) = parse_length(buf, Self::PREFIX)?;
        Ok(end + (CRLF_LEN * 2) + len)
    }
}

// bool 数据类型的模型
// #<t|f>\r\n
impl RespDecode for bool {
    const PREFIX: &'static str = "#";
    fn decode(buf: &mut BytesMut) -> Result<Self, RespDecodeError> {
        match extract_fixed_data(buf, "#t\r\n", "Bool") {
            Ok(_) => Ok(true),
            Err(_) => match extract_fixed_data(buf, "#f\r\n", "Bool") {
                Ok(_) => Ok(false),
                Err(e) => Err(e),
            },
        }
    }

    fn expect_length(_buf: &[u8]) -> Result<usize, RespDecodeError> {
        Ok(4)
    }
}

// Array
// 	- *<number-of-elements>\r\n<element-1>...<element-n>
// - "*2\r\n$3\r\nget\r\n$5\r\nhello\r\n"
impl RespDecode for RespArray {
    const PREFIX: &'static str = "*";
    fn decode(buf: &mut BytesMut) -> Result<Self, RespDecodeError> {
        let (end, len) = parse_length(buf, Self::PREFIX)?;

        let total_len = calc_total_length(buf, end, len, Self::PREFIX)?;

        if buf.len() < total_len {
            return Err(RespDecodeError::NotComplete);
        }
        // 裁切掉prefix + \r\n 部分
        buf.advance(end + CRLF_LEN);
        let mut frames = Vec::with_capacity(len);
        for _ in 0..len {
            frames.push(RespFrame::decode(buf)?);
        }
        Ok(RespArray::new(frames))
    }
    // Array 的 total len应该需要将每个元素加起来，累积在一起。
    fn expect_length(buf: &[u8]) -> Result<usize, RespDecodeError> {
        let (end, len) = parse_length(buf, Self::PREFIX)?;
        calc_total_length(buf, end, len, Self::PREFIX)
    }
}

// - ,[<+|->]<integral>[.<fractional>][<E|e>[sign]<exponent>]\r\n
impl RespDecode for f64 {
    const PREFIX: &'static str = ",";
    fn decode(buf: &mut BytesMut) -> Result<Self, RespDecodeError> {
        let end = extract_simple_frame_data(buf, Self::PREFIX)?;
        // 这里需要把buf消耗掉，否则后续使用时，会把前面的数据累加在上面
        let data = buf.split_to(end + 2);
        let s = String::from_utf8_lossy(&data[1..end]);
        let v = s.parse()?;
        Ok(v)
    }

    fn expect_length(buf: &[u8]) -> Result<usize, RespDecodeError> {
        let end = extract_simple_frame_data(buf, Self::PREFIX)?;
        Ok(end + CRLF_LEN)
    }
}
// Map
// - %<number-of-entries>\r\n<key-1><value-1>...<key-n><value-n>
// %2\r\n+hello\r\n$5\r\nworld\r\n+foo\r\n$3\r\nbar\r\n
impl RespDecode for RespMap {
    const PREFIX: &'static str = "%";
    fn decode(buf: &mut BytesMut) -> Result<Self, RespDecodeError> {
        let (end, len) = parse_length(buf, Self::PREFIX)?;
        let total_len = calc_total_length(buf, end, len, Self::PREFIX)?;

        if buf.len() < total_len {
            return Err(RespDecodeError::NotComplete);
        }
        // 裁切
        buf.advance(end + CRLF_LEN);

        let mut frames = RespMap::new();
        for _ in 0..len {
            // 这里我们认为所有的key都是 SimpleString
            let key = SimpleString::decode(buf)?;
            // 所有的value就根据 prefix进行动态获取
            let value = RespFrame::decode(buf)?;
            frames.insert(key.0, value);
        }
        Ok(frames)
    }

    fn expect_length(buf: &[u8]) -> Result<usize, RespDecodeError> {
        let (end, len) = parse_length(buf, Self::PREFIX)?;
        calc_total_length(buf, end, len, Self::PREFIX)
    }
}

// Set
// - ~<number-of-elements>\r\n<element-1>...<element-n>
impl RespDecode for RespSet {
    const PREFIX: &'static str = "~";
    fn decode(buf: &mut BytesMut) -> Result<Self, RespDecodeError> {
        let (end, len) = parse_length(buf, Self::PREFIX)?;
        let total_len = calc_total_length(buf, end, len, Self::PREFIX)?;
        if buf.len() < total_len {
            return Err(RespDecodeError::NotComplete);
        }
        // 裁切
        buf.advance(end + CRLF_LEN);

        let mut frames = Vec::new();
        for _ in 0..len {
            let temp = RespFrame::decode(buf)?;
            frames.push(temp);
        }
        Ok(RespSet::new(frames))
    }

    fn expect_length(buf: &[u8]) -> Result<usize, RespDecodeError> {
        let (end, len) = parse_length(buf, Self::PREFIX)?;
        calc_total_length(buf, end, len, Self::PREFIX)
    }
}

fn extract_fixed_data(
    buf: &mut BytesMut,
    expected: &str,
    data_type: &str,
) -> Result<usize, RespDecodeError> {
    if buf.len() < 2 {
        return Err(RespDecodeError::NotComplete);
    }
    // 检查是否匹配
    if !buf.starts_with(expected.as_bytes()) {
        return Err(RespDecodeError::InvalidFrameType(format!(
            "except: {}, got:{:?}",
            data_type, buf
        )));
    }
    println!(
        "===前缀:{}, buf的内容={:?}",
        expected,
        String::from_utf8_lossy(buf)
    );
    let mut end = 0;
    // 检查类型如果是 Bool类型，则根据数据格式，从buf中取出内容
    if data_type.eq("Bool") {
        for i in 0..buf.len() - 1 {
            if buf[i] == b'\r' && buf[i + 1] == b'\n' {
                // 获取到\r的下标
                end = i;
                break;
            }
        }
        // 移除buf中的数据
        let _ = buf.split_to(end + 2);
    }
    // 判定end不为0，否则抛出异常
    if end == 0 {
        return Err(RespDecodeError::NotComplete);
    }
    Ok(end)
}

// 增加额外参数，用于传入前缀（如"+" 或 "-"）
fn extract_simple_frame_data(buf: &[u8], prefix: &str) -> Result<usize, RespDecodeError> {
    if buf.len() < 3 {
        return Err(RespDecodeError::NotComplete);
    }
    // 检查是否以指定的前缀开始 这里由于后续其他地方也需要使用，因此
    // 我们需要让它变得更为通用
    // if !buf.starts_with(prefix.as_bytes()) {
    //     return Err(RespDecodeError::InvalidFrameType(format!("except: SimpleString(+), got:{:?}", buf)));
    // }
    if !buf.starts_with(prefix.as_bytes()) {
        return Err(RespDecodeError::InvalidFrameType(format!(
            "expect: prefix({}), got: {:?}",
            prefix, buf
        )));
    }
    // simple String +OK\r\n
    // search for "\r\n"
    /* let mut end = 0;
    for i in 0..buf.len() - 1 {
        if buf[i] == b'\r' && buf[i + 1] == b'\n' {
            // 获取到\r的下标
            end = i;
            break;
        }
    } */
    let end = find_crlf(buf, 1).ok_or(RespDecodeError::NotComplete)?;
    // 判定end不为0，否则抛出异常
    /* if end == 0 {
        return Err(RespDecodeError::NotComplete);
    } */
    Ok(end)
}

/**
 * 根据prefix 获取buf的 结束值和内容的长度
 */
fn parse_length(bytes_buf: &[u8], prefix: &str) -> Result<(usize, usize), RespDecodeError> {
    // let mut bytes_buf = BytesMut::new();
    // bytes_buf.extend_from_slice(buf);
    let end = extract_simple_frame_data(bytes_buf, prefix)?;
    let s = String::from_utf8_lossy(&bytes_buf[prefix.len()..end]);
    Ok((end, s.parse()?))
}

// 检测 需要decode的数据，是否满足格式
fn calc_total_length(
    buf: &[u8],
    end: usize,
    len: usize,
    prefix: &str,
) -> Result<usize, RespDecodeError> {
    // 假设一切都是美好的
    let mut total = end + CRLF_LEN; //此处是， array, map, set 类型去掉前缀之后的end + \r\n 的长度。
    let mut data = &buf[total..]; // 获取去掉 前缀开始，到整个buf的全部内容
    let tmp = String::from_utf8_lossy(data);
    println!("测试结果={:?}", tmp);
    match prefix {
        // array and set of prefix
        // *<number-of-elements>\r\n<element-1>...<element-n>
        // ~<number-of-elements>\r\n<element-1>...<element-n>
        // 对于 array 和 set 而言，
        "*" | "~" => {
            // this CRLF in the buffer, for array and set , we need to find 1 CRLF for each element
            // find_crlf(data, len)
            //     .map(|end| len + CRLF_LEN + end)
            //     .ok_or(RespDecodeError::NotComplete)
            for _ in 0..len {
                // 针对不同类型，获取相应的 item_len
                let item_len = RespFrame::expect_length(data)?;
                // 根据length，截取对应长度的数据到data
                data = &data[item_len..]; //data[len..] 意味着创建从 len 索引位置开始（包含 len 索引位置的元素），直到 data 末尾的切片。
                total += item_len;
            }
            Ok(total)
        }
        // %开头，是一个map
        // %<number-of-entries>\r\n<key-1><value-1>...<key-n><value-n>
        // 对于 需要 找到两个 CRLF 进行 each key-value 的键值对
        "%" => {
            // find_crlf(data, len * 2)
            // .map(|end| len + CRLF_LEN + end)
            // .ok_or(RespDecodeError::NotComplete)

            for _ in 0..len {
                // fist map the key is SimpleString type.
                let key_len = SimpleString::expect_length(data)?;
                data = &data[key_len..];
                total += key_len;

                // second map the value is any RespFrame.
                let value_len = RespFrame::expect_length(data)?;
                data = &data[value_len..];
                total += value_len;
            }
            Ok(total)
        }
        _ => Ok(len + CRLF_LEN),
    }
}

fn find_crlf(buf: &[u8], nth: usize) -> Option<usize> {
    let mut count = 0;
    for i in 1..buf.len() - 1 {
        if buf[i] == b'\r' && buf[i + 1] == b'\n' {
            count += 1;
            if count == nth {
                return Some(i);
            }
        }
    }
    None
}

/// 进行单元测试
#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::{Ok, Result};
    use bytes::{Buf, BufMut};
    #[test]
    fn test_simple_string_decode() -> Result<()> {
        // 设置测试样本
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"+OK\r\n");

        let frame = SimpleString::decode(&mut buf)?;
        assert_eq!(frame, SimpleString::new("OK".to_string()));

        buf.extend_from_slice(b"+hello\r"); //由于这一步没有获取成功，因此buf里面的东西还在
        let ret = SimpleString::decode(&mut buf);
        let _ = format!("except: SimpleString(+), got:{:?}", buf);
        assert_eq!(ret.unwrap_err(), RespDecodeError::NotComplete);

        buf.put_u8(b'\n'); //此处给它追加，因此，就得到了一个完整的 "+hello\r\n"
        let frame = SimpleString::decode(&mut buf)?;
        assert_eq!(frame, SimpleString::new("hello".to_string()));

        Ok(())
    }

    #[test]
    fn test_bulk_string_decode() -> Result<()> {
        // 设置测试样本
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"$5\r\nhello\r\n");

        let frame = BulkString::decode(&mut buf)?;
        assert_eq!(frame, BulkString::new(b"hello"));
        Ok(())
    }

    #[test]
    // SimpleError
    fn test_simple_error_decode() -> Result<()> {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"-Error message\r\n");

        let frame = SimpleError::decode(&mut buf)?;
        assert_eq!(frame, SimpleError::new("Error message".to_string()));

        buf.extend_from_slice(b"-hello\r");
        let ret = SimpleError::decode(&mut buf);
        assert_eq!(ret.unwrap_err(), RespDecodeError::NotComplete);

        buf.put_u8(b'\n');
        let ret = SimpleError::decode(&mut buf)?;
        assert_eq!(ret, SimpleError::new("hello".to_string()));

        Ok(())
    }

    #[test]
    fn test_integers_decode() -> Result<()> {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b":123\r\n");

        let frame = i64::decode(&mut buf)?;
        assert_eq!(frame, 123);

        buf.extend_from_slice(b":-456\r\n");
        let frame = i64::decode(&mut buf)?;
        assert_eq!(frame, -456);

        buf.extend_from_slice(b":0\r\n");
        let frame = i64::decode(&mut buf)?;
        assert_eq!(frame, 0);

        buf.extend_from_slice(b":1234567890123\r\n");
        let frame = i64::decode(&mut buf)?;
        assert_eq!(frame, 1234567890123);

        Ok(())
    }

    #[test]
    fn test_bool_decode() {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"#t\r\n");

        let frame = bool::decode(&mut buf).unwrap();
        assert!(frame);

        buf.extend_from_slice(b"#f\r\n");
        let frame = bool::decode(&mut buf).unwrap();
        assert!(!frame);

        buf.extend_from_slice(b"#true\r\n");
        let ret = bool::decode(&mut buf);
        assert_eq!(
            ret.unwrap_err(),
            RespDecodeError::InvalidFrameType("except: Bool, got:b\"#true\\r\\n\"".to_string())
        );

        // 由于decode失败，不会清空buf中的内容，需要重新测试，则要手动清理一下
        buf.advance(buf.len());

        buf.extend_from_slice(b"#false\r\n");
        let frame = bool::decode(&mut buf);
        assert_eq!(
            frame.unwrap_err(),
            RespDecodeError::InvalidFrameType("except: Bool, got:b\"#false\\r\\n\"".to_string())
        );
    }

    #[test]
    fn test_array_decode() -> Result<()> {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"*2\r\n$3\r\nset\r\n$5\r\nhello\r\n");

        let frame = RespArray::decode(&mut buf)?;
        assert_eq!(
            frame,
            RespArray::new(vec![
                BulkString::new(b"set".to_vec()).into(),
                BulkString::new(b"hello".to_vec()).into(),
            ])
        );

        buf.extend_from_slice(b"*2\r\n$3\r\nset\r\n");
        let ret = RespArray::decode(&mut buf);
        assert_eq!(ret.unwrap_err(), RespDecodeError::NotComplete);

        buf.extend_from_slice(b"$5\r\nhello\r\n");
        let frame = RespArray::decode(&mut buf)?;
        assert_eq!(frame, RespArray::new([b"set".into(), b"hello".into()]));
        Ok(())
    }

    #[test]
    fn test_double_decode() -> Result<()> {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b",123.45\r\n");

        let frame = f64::decode(&mut buf)?;
        assert_eq!(frame, 123.45);

        buf.extend_from_slice(b",+1.23456e-9\r\n");
        let frame = f64::decode(&mut buf)?;
        assert_eq!(frame, 1.23456e-9);

        Ok(())
    }

    //
    #[test]
    fn test_map_decode() -> Result<()> {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"%2\r\n+hello\r\n$5\r\nworld\r\n+foo\r\n$3\r\nbar\r\n");

        let frame = RespMap::decode(&mut buf)?;
        let mut map = RespMap::new();
        map.insert(
            "hello".to_string(),
            BulkString::new(b"world".to_vec()).into(),
        );
        map.insert("foo".to_string(), BulkString::new(b"bar".to_vec()).into());
        assert_eq!(frame, map);

        Ok(())
    }

    #[test]
    fn dymmy_test() {
        let s = "+1.23456e-9";
        let f: f64 = s.parse().unwrap();
        assert_eq!(f, 1.23456e-9);
    }

    #[test]
    fn test_calc_array_length() -> Result<()> {
        let buf = b"*2\r\n$3\r\nset\r\n$5\r\nhello\r\n";
        let (end, len) = parse_length(buf, "*")?;
        let total_len = calc_total_length(buf, end, len, "*")?;

        assert_eq!(total_len, buf.len());

        let buf = b"*2\r\n$3\r\nset\r\n";
        let (end, len) = parse_length(buf, "*")?;
        let ret = calc_total_length(buf, end, len, "*");
        assert_eq!(ret.unwrap_err(), RespDecodeError::NotComplete);

        Ok(())
    }
}
