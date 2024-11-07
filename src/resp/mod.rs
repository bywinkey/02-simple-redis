mod decode;
mod encode;

use bytes::BytesMut;
use enum_dispatch::enum_dispatch;
use std::{
    collections::BTreeMap,
    ops::{Deref, DerefMut},
};
use thiserror::Error;
/*
Simple strings
    - +OK\r\n
Simple errors
    - -Error message\r\n
Bulk errors
    - !<length>\r\n<error>\r\n
Integers
    - :[<+|->]<value>\r\n
Bulk strings
    - $<length>\r\n<data>\r\n
Null bulk strings
    - $-1\r\n
Arrays
    - *<number-of-elements>\r\n<element-1>...<element-n>
Null arrays
    - *-1\r\n
Nulls
    - _\r\n
Booleans
    - #<t|f>\r\n
Doubles
    - ,[<+|->]<integral>[.<fractional>][<E|e>[sign]<exponent>]\r\n
Maps
    - %<number-of-entries>\r\n<key-1><value-1>...<key-n><value-n>
Sets
    - ~<number-of-elements>\r\n<element-1>...<element-n>
 */

/// 将输入的类型转换成Vec<u8>
#[enum_dispatch]
pub trait RespEncode {
    fn encode(self) -> Vec<u8>;
}

/// 解码，将byte类型的数据转化成一个RespFrame
pub trait RespDecode: Sized {
    // 描述类型的协议开头
    const PREFIX: &'static str;
    fn decode(buf: &mut BytesMut) -> Result<Self, RespDecodeError>;
    // 针对不同类型，获取 预期 的len
    fn expect_length(buf: &[u8]) -> Result<usize, RespDecodeError>;
}

#[derive(Debug, Error, PartialEq)]
pub enum RespDecodeError {
    // #[error("Invalid frame :{0}")]
    // InvalidFrame(String),
    #[error("Invalid frame type :{0}")]
    InvalidFrameType(String),
    // #[error("Invalid frame length :{0}")]
    // InvalidFrameLength(isize),
    #[error("Frame is not Complete")]
    NotComplete,
    #[error("Parse error: {0}")]
    ParseIntError(#[from] std::num::ParseIntError),
    #[error("UTF8 error: {0}")]
    Utf8Error(#[from] std::str::Utf8Error),
    #[error("Parse float error: {0}")]
    ParseFloatError(#[from] std::num::ParseFloatError),
}

#[enum_dispatch(RespEncode)]
#[derive(Debug, PartialEq)]
pub enum RespFrame {
    SimpleString(SimpleString),
    Error(SimpleError),
    Integer(i64),
    BulkString(BulkString),
    NullBulkString(RespNullBulkString),
    Array(RespArray),
    Null(RespNull),
    NullArray(RespNullArray),
    Boolean(bool),
    Double(f64),
    Map(RespMap),
    Set(RespSet),
}
#[derive(Debug, PartialEq, Eq)]
pub struct SimpleString(String);

#[derive(Debug, PartialEq, Eq)]
pub struct SimpleError(String);

#[derive(Debug, PartialEq, Eq)]
pub struct BulkString(Vec<u8>);

#[derive(Debug, PartialEq, Eq)]
pub struct RespNullBulkString;

#[derive(Debug, PartialEq)]
pub struct RespArray(Vec<RespFrame>);

#[derive(Debug, PartialEq, Eq)]
pub struct RespNull;
#[derive(Debug, PartialEq, Eq)]
pub struct RespNullArray;

#[derive(Debug, PartialEq)]
pub struct RespMap(BTreeMap<String, RespFrame>);

#[derive(Debug, PartialEq)]
pub struct RespSet(Vec<RespFrame>);

impl Deref for SimpleString {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Deref for SimpleError {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Deref for RespArray {
    type Target = Vec<RespFrame>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Deref for RespMap {
    type Target = BTreeMap<String, RespFrame>;

    fn deref(&self) -> &Self::Target {
        // 元组数据取值方式
        &self.0
    }
}

// 解引用操作，默认的HashMap是不可变的，这里通过DerefMut将其修饰为可变的
impl DerefMut for RespMap {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Deref for RespSet {
    type Target = Vec<RespFrame>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl SimpleString {
    pub fn new(s: impl Into<String>) -> Self {
        SimpleString(s.into())
    }
}

impl SimpleError {
    pub fn new(s: impl Into<String>) -> Self {
        SimpleError(s.into())
    }
}

impl BulkString {
    pub fn new(b: impl Into<Vec<u8>>) -> Self {
        BulkString(b.into())
    }
}

impl RespArray {
    pub fn new(frames: impl Into<Vec<RespFrame>>) -> Self {
        RespArray(frames.into())
    }
}

impl RespMap {
    pub fn new() -> Self {
        RespMap(BTreeMap::new())
    }
}

impl RespSet {
    pub fn new(s: impl Into<Vec<RespFrame>>) -> Self {
        RespSet(s.into())
    }
}

// impl From<&[u8]> for BulkString {
//     fn from(value: &[u8]) -> Self {
//         BulkString(value.into())
//     }
// }

// impl From<Vec<u8>> for BulkString {
//     fn from(value: Vec<u8>) -> Self {
//         BulkString(value.into())
//     }
// }

impl From<&str> for RespFrame {
    fn from(value: &str) -> Self {
        SimpleString(value.to_string()).into()
    }
}

impl From<&[u8]> for RespFrame {
    fn from(value: &[u8]) -> Self {
        BulkString(value.into()).into()
    }
}

// impl<const N: usize> From<&[u8; N]> for BulkString {
//     fn from(s: &[u8; N]) -> Self {
//         BulkString(s.to_vec())
//     }
// }

impl<const N: usize> From<&[u8; N]> for RespFrame {
    fn from(s: &[u8; N]) -> Self {
        BulkString(s.to_vec()).into()
    }
}
