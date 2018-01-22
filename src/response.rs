//! Response types

use std::collections::HashMap;
use std::ops::Not;

/// Representation of API Gateway response
///
/// # Examples
///
/// ```
/// # use gateway::Response;
/// let response: Response = Response::default();
/// assert!(response.status_code() == 200);
/// assert!(response.headers().is_empty());
/// assert!(response.body() == "");
/// ```
#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Response {
    status_code: u16,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    headers: HashMap<String, String>,
    #[serde(skip_serializing_if = "String::is_empty")]
    body: String,
    #[serde(skip_serializing_if = "Not::not")]
    is_base64_encoded: bool,
}

impl Default for Response {
    fn default() -> Self {
        Response::builder().build()
    }
}

impl Response {
    pub fn builder() -> Builder {
        Builder::new()
    }

    pub fn status_code(&self) -> u16 {
        self.status_code
    }

    pub fn headers(&self) -> &HashMap<String, String> {
        &self.headers
    }

    pub fn body(&self) -> &String {
        &self.body
    }
}

/// An HTTP response builder
#[derive(Debug, Default)]
pub struct Builder {
    status_code: u16,
    headers: HashMap<String, String>,
    body: String,
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

    pub fn body<B>(&mut self, b: B) -> &mut Self
    where
        B: Into<String>,
    {
        self.body = b.into();
        self
    }

    pub fn header<K, V>(&mut self, k: K, v: V) -> &mut Self
    where
        K: Into<String>,
        V: Into<String>,
    {
        self.headers.insert(k.into(), v.into());
        self
    }

    pub fn is_base64_encoded(&mut self, is: bool) -> &mut Self {
        self.is_base64_encoded = is;
        self
    }

    pub fn build(&self) -> Response {
        Response {
            status_code: self.status_code,
            headers: self.headers.clone(),
            body: self.body.clone(),
            is_base64_encoded: self.is_base64_encoded,
        }
    }
}

#[cfg(test)]
mod tests {

    use Response;
    use serde_json;

    #[test]
    fn default_response() {
        assert_eq!(Response::default().status_code, 200)
    }
    #[test]
    fn builder_body() {
        assert_eq!(Response::builder().body("foo").build().body, "foo")
    }
    #[test]
    fn serialize_default() {
        assert_eq!(
            serde_json::to_string(&Response::default()).expect("failed to serialize response"),
            r#"{"statusCode":200}"#
        );
    }

    #[test]
    fn serialize_body() {
        assert_eq!(
            serde_json::to_string(&Response::builder().body("foo").build())
                .expect("failed to serialize response"),
            r#"{"statusCode":200,"body":"foo"}"#
        );
    }
}