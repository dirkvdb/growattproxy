#![warn(clippy::unwrap_used)]
pub mod dataprocessor;
pub mod layouts;
pub mod mqtt;
pub mod proxy;

#[cfg(feature = "sniffer")]
pub mod sniffer;

use std::{array::TryFromSliceError, fmt, io::Write, net::AddrParseError, path::Path, str::Utf8Error};

pub fn dump_packet(data: &[u8], output: &Path) -> Result<(), ProxyError> {
    let mut file = std::fs::OpenOptions::new().write(true).create(true).open(output)?;
    file.write_all(&data)?;

    Ok(())
}

#[derive(Debug)]
pub enum ProxyError {
    NetworkError(String),
    RuntimeError(String),
    ParseError,
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

impl From<Utf8Error> for ProxyError {
    fn from(_: Utf8Error) -> Self {
        ProxyError::ParseError
    }
}

impl From<rumqttc::ClientError> for ProxyError {
    fn from(err: rumqttc::ClientError) -> Self {
        ProxyError::RuntimeError(format!("MQTT Error: {err}"))
    }
}

impl From<rumqttc::ConnectionError> for ProxyError {
    fn from(err: rumqttc::ConnectionError) -> Self {
        ProxyError::RuntimeError(format!("MQTT Connection error: {err}"))
    }
}
