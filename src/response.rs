//! Response types

use std::collections::HashMap;
use std::ops::Not;

/// Representation of API Gateway response
///
/// # Examples
#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub(crate) struct GatewayResponse {
    pub status_code: u16,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub headers: HashMap<String, String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
    #[serde(skip_serializing_if = "Not::not")]
    pub is_base64_encoded: bool,
}

impl Default for GatewayResponse {
    fn default() -> Self {
        Self {
            status_code: 200,
            headers: Default::default(),
            body: Default::default(),
            is_base64_encoded: Default::default(),
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
    fn serialize_default() {
        assert_eq!(
            serde_json::to_string(&GatewayResponse::default())
                .expect("failed to serialize response"),
            r#"{"statusCode":200}"#
        );
    }

    #[test]
    fn serialize_body() {
        let mut resp = GatewayResponse::default();
        resp.body = Some("foo".into());
        assert_eq!(
            serde_json::to_string(&resp).expect("failed to serialize response"),
            r#"{"statusCode":200,"body":"foo"}"#
        );
    }
}
