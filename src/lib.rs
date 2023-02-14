#![warn(clippy::unwrap_used)]
pub mod proxy;
pub mod dataprocessor;

use std::{fmt, net::AddrParseError, array::TryFromSliceError};

#[derive(Debug)]
pub enum ProxyError {
    NetworkError(String),
    RuntimeError(String),
    ParseError
}

impl fmt::Display for ProxyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ProxyError::NetworkError(str) => write!(f, "Network Error {}", str),
            ProxyError::RuntimeError(str) => write!(f, "Runtime Error {}", str),
            ProxyError::ParseError => write!(f, "Parse Error"),
        }
    }
}

impl From<std::io::Error> for ProxyError {
    fn from(err: std::io::Error) -> Self {
        ProxyError::NetworkError(format!("IO: {}", err))
    }
}


impl From<std::num::ParseIntError> for ProxyError {
    fn from(_: std::num::ParseIntError) -> Self {
        ProxyError::ParseError
    }
}

impl From<String> for ProxyError {
    fn from(str: String) -> Self {
        ProxyError::RuntimeError(str)
    }
}

impl From<AddrParseError> for ProxyError {
    fn from(_: AddrParseError) -> Self {
        ProxyError::NetworkError(String::from("Invalid address"))
    }
}

impl From<TryFromSliceError> for ProxyError {
    fn from(_: TryFromSliceError) -> Self {
        ProxyError::ParseError
    }
}