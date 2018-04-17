//! Response types

use std::collections::HashMap;
use std::ops::Not;

/// Representation of API Gateway response
///
/// # Examples
#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub(crate) struct GatewayResponse {
    status_code: u16,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    headers: HashMap<String, String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    body: Option<String>,
    #[serde(skip_serializing_if = "Not::not")]
    is_base64_encoded: bool,
}

impl Default for GatewayResponse {
    fn default() -> Self {
        Self::builder().build()
    }
}

impl GatewayResponse {
    pub fn builder() -> Builder {
        Builder::new()
    }
}

/// An HTTP response builder
///
/// You will typically should create one with `GatewayResponse::builder()`
#[derive(Debug, Default)]
pub(crate) struct Builder {
    status_code: u16,
    headers: HashMap<String, String>,
    body: Option<String>,
    is_base64_encoded: bool,
}

impl Builder {
    pub fn new() -> Self {
        Self {
            status_code: 200,
            ..Default::default()
        }
    }

    pub fn status_code(&mut self, c: u16) -> &mut Self {
        self.status_code = c;
        self
    }

    /// Set response body. If body is base64 encoded
    /// set `base64_encoded(true)`.
    pub fn body<B>(&mut self, b: B) -> &mut Self
    where
        B: Into<Option<String>>,
    {
        self.body = b.into();
        self
    }

    pub fn headers<K, V>(&mut self, hdrs: Vec<(K, V)>) -> &mut Self
    where
        K: Into<String>,
        V: Into<String>,
    {
        for (k, v) in hdrs {
            self.headers.insert(k.into(), v.into());
        }
        self
    }

    /// Sets a hint to API gateway that response body is base64 encoded
    pub fn base64_encoded(&mut self, is: bool) -> &mut Self {
        self.is_base64_encoded = is;
        self
    }

    pub fn build(&self) -> GatewayResponse {
        GatewayResponse {
            status_code: self.status_code,
            headers: self.headers.clone(),
            body: self.body.clone(),
            is_base64_encoded: self.is_base64_encoded,
        }
    }
}

#[cfg(test)]
mod tests {

    use super::GatewayResponse;
    use serde_json;

    #[test]
    fn default_response() {
        assert_eq!(GatewayResponse::default().status_code, 200)
    }
    #[test]
    fn builder_body() {
        assert_eq!(
            GatewayResponse::builder()
                .body("foo".to_owned())
                .build()
                .body,
            Some("foo".to_owned())
        )
    }
    #[test]
    fn serialize_default() {
        assert_eq!(
            serde_json::to_string(&GatewayResponse::default())
                .expect("failed to serialize response"),
            r#"{"statusCode":200}"#
        );
    }

    #[test]
    fn serialize_body() {
        assert_eq!(
            serde_json::to_string(&GatewayResponse::builder().body("foo".to_owned()).build())
                .expect("failed to serialize response"),
            r#"{"statusCode":200,"body":"foo"}"#
        );
    }
}
