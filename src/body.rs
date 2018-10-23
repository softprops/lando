//! provides a http request nad response body entity interface

// Std
use std::borrow::Cow;
use std::ops::Deref;

// Third Party
use base64::display::Base64Display;
use bytes::Bytes;
use serde::ser::{Error as SerError, Serialize, Serializer};

/// Representation of http request and response bodies as supported
/// by API Gateway.
///
/// These come in three flavors `Empty` ( to body ), `Text` ( text data ), `Binary` ( binary data ).
///
/// Body types can be `Deref` and `AsRef`'d into `[u8]` types much like the `hyper` crate
///
/// # Examples
///
/// Body types are inverted with `From` implementations. Types like `String`, `str` whose type reflects
/// text produce `Body::Text` variants
///
/// ```
/// assert!(match lando::Body::from("text") {
///   lando::Body::Text(_) => true,
///   _ => false
/// })
/// ```
///
/// Types like `Vec<u8>` and `&[u8]` whose types reflect raw bytes produce `Body::Binary` variants
///
/// ```
/// assert!(match lando::Body::from("text".as_bytes()) {
///   lando::Body::Binary(_) => true,
///   _ => false
/// })
/// ```
///
/// The unit type ( `()` ) whose type represents an empty value produces `Body::Empty` variants
///
/// ```
/// assert!(match lando::Body::from(()) {
///   lando::Body::Empty => true,
///   _ => false
/// })
/// ```
///
///
/// For more information about API Gateway's body types,
/// refer to [this documentation](https://docs.aws.amazon.com/apigateway/latest/developerguide/api-gateway-payload-encodings.html).
#[derive(Debug, PartialEq)]
pub enum Body {
    /// An empty body
    Empty,
    /// A body containing string data
    Text(Bytes),
    /// A body containing binary data
    Binary(Bytes),
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

impl<'a> From<&'a str> for Body {
    fn from(s: &'a str) -> Self {
        Body::Text(Bytes::from(s))
    }
}

impl From<String> for Body {
    fn from(b: String) -> Self {
        Body::Text(Bytes::from(b))
    }
}

impl From<Cow<'static, str>> for Body {
    #[inline]
    fn from(cow: Cow<'static, str>) -> Body {
        match cow {
            Cow::Borrowed(b) => Body::from(b),
            Cow::Owned(o) => Body::from(o),
        }
    }
}

impl From<Cow<'static, [u8]>> for Body {
    #[inline]
    fn from(cow: Cow<'static, [u8]>) -> Body {
        match cow {
            Cow::Borrowed(b) => Body::from(b),
            Cow::Owned(o) => Body::from(o),
        }
    }
}

impl From<Vec<u8>> for Body {
    fn from(b: Vec<u8>) -> Self {
        Body::Binary(Bytes::from(b))
    }
}

impl<'a> From<&'a [u8]> for Body {
    fn from(b: &'a [u8]) -> Self {
        Body::Binary(Bytes::from(b))
    }
}

impl Deref for Body {
    type Target = [u8];

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl AsRef<[u8]> for Body {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        match self {
            Body::Empty => &[],
            Body::Text(ref bytes) => bytes,
            Body::Binary(ref bytes) => bytes,
        }
    }
}

impl<'a> Serialize for Body {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Body::Text(data) => {
                serializer.serialize_str(::std::str::from_utf8(data).map_err(S::Error::custom)?)
            }
            Body::Binary(data) => serializer.collect_str(&Base64Display::standard(data)),
            Body::Empty => serializer.serialize_unit(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;
    use std::collections::HashMap;

    #[test]
    fn body_has_default() {
        assert_eq!(Body::default(), Body::Empty);
    }

    #[test]
    fn from_unit() {
        assert_eq!(Body::from(()), Body::Empty);
    }

    #[test]
    fn from_str() {
        match Body::from(String::from("foo").as_str()) {
            Body::Text(_) => (),
            not => assert!(false, "expected Body::Text(...) got {:?}", not),
        }
    }

    #[test]
    fn from_string() {
        match Body::from(String::from("foo")) {
            Body::Text(_) => (),
            not => assert!(false, "expected Body::Text(...) got {:?}", not),
        }
    }

    #[test]
    fn from_cow_str() {
        match Body::from(Cow::from("foo")) {
            Body::Text(_) => (),
            not => assert!(false, "expected Body::Text(...) got {:?}", not),
        }
    }

    #[test]
    fn from_cow_bytes() {
        match Body::from(Cow::from("foo".as_bytes())) {
            Body::Binary(_) => (),
            not => assert!(false, "expected Body::Binary(...) got {:?}", not),
        }
    }

    #[test]
    fn from_bytes() {
        match Body::from("foo".as_bytes()) {
            Body::Binary(_) => (),
            not => assert!(false, "expected Body::Binary(...) got {:?}", not),
        }
    }

    #[test]
    fn serialize_text() {
        let mut map = HashMap::new();
        map.insert("foo", Body::from("bar"));
        assert_eq!(serde_json::to_string(&map).unwrap(), r#"{"foo":"bar"}"#);
    }

    #[test]
    fn serialize_binary() {
        let mut map = HashMap::new();
        map.insert("foo", Body::from("bar".as_bytes()));
        assert_eq!(serde_json::to_string(&map).unwrap(), r#"{"foo":"YmFy"}"#);
    }

    #[test]
    fn serialize_empty() {
        let mut map = HashMap::new();
        map.insert("foo", Body::Empty);
        assert_eq!(serde_json::to_string(&map).unwrap(), r#"{"foo":null}"#);
    }
}
