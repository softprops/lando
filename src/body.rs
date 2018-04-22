//! provides a http request nad response body entity interface

use bytes::Bytes;

/// Variations of requests and response bodies
#[derive(Debug)]
pub enum Body {
  Empty,
  Bytes(Bytes)
}

impl Default for Body {
  fn default() -> Self {
    Body::Empty
  }
}

impl From<()> for Body {
  fn from(_: ()) -> Self {
    Body::Empty
  }
}

impl <'a> From<&'a str> for Body {
  fn from(s: &'a str) -> Self {
    Body::Bytes(Bytes::from(s))
  }
}

impl From<Vec<u8>> for Body {
  fn from(b: Vec<u8>) -> Self {
    Body::Bytes(Bytes::from(b))
  }
}

impl <'a> From<&'a [u8]> for Body {
  fn from(b: &'a [u8]) -> Self {
    Body::Bytes(Bytes::from(b))
  }
}

impl From<String> for Body {
  fn from(b: String) -> Self {
    Body::Bytes(Bytes::from(b))
  }
}