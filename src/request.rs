//! API Gateway request types. Typically these are exposed via the `request_context` method provided by [lando::RequestExt](trait.RequestExt.html)

// Std
use std::collections::HashMap;

// Third Party
use serde::{Deserialize, Deserializer};

/// Representation of API Gateway proxy event data
/// this should really be pub(crate) but is bump for
/// bench mark testing
#[doc(hidden)]
#[derive(Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct GatewayRequest {
    //pub resource: String,
    pub path: String,
    pub http_method: String,
    pub headers: HashMap<String, String>,
    #[serde(deserialize_with = "nullable_default")]
    pub query_string_parameters: HashMap<String, String>,
    #[serde(deserialize_with = "nullable_default")]
    pub path_parameters: HashMap<String, String>,
    #[serde(deserialize_with = "nullable_default")]
    pub stage_variables: HashMap<String, String>,
    pub body: Option<String>,
    #[serde(default)]
    pub is_base64_encoded: bool,
    pub request_context: RequestContext,
}

/// API Gateway request context
#[derive(Deserialize, Debug, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RequestContext {
    //pub path: String,
    pub account_id: String,
    pub resource_id: String,
    pub stage: String,
    pub request_id: String,
    pub resource_path: String,
    pub http_method: String,
    //pub authorizer: HashMap<String, String>,
    pub api_id: String,
    pub identity: Identity,
}

/// Identity assoicated with request
#[derive(Deserialize, Debug, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Identity {
    pub source_ip: String,
    pub cognito_identity_id: Option<String>,
    pub cognito_identity_pool_id: Option<String>,
    pub cognito_authentication_provider: Option<String>,
    pub cognito_authentication_type: Option<String>,
    pub account_id: Option<String>,
    pub caller: Option<String>,
    pub api_key: Option<String>,
    pub user: Option<String>,
    pub user_agent: Option<String>,
    pub user_arn: Option<String>,
}

/// deserializes (json) null values to their default values
// https://github.com/serde-rs/serde/issues/1098
fn nullable_default<'de, T, D>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: Default + Deserialize<'de>,
{
    let opt = Option::deserialize(deserializer)?;
    Ok(opt.unwrap_or_else(Default::default))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;
    use std::collections::HashMap;

    #[test]
    fn deserializes_request_events() {
        // from the docs
        // https://docs.aws.amazon.com/lambda/latest/dg/eventsources.html#eventsources-api-gateway-request
        let input = include_str!("../tests/data/proxy_request.json");
        assert!(serde_json::from_str::<GatewayRequest>(&input).is_ok())
    }

    #[test]
    fn implements_default() {
        assert_eq!(
            GatewayRequest {
                path: "/foo".into(),
                ..Default::default()
            }
            .path,
            "/foo"
        )
    }

    #[test]
    fn deserialize_with_null() {
        #[derive(Debug, PartialEq, Deserialize)]
        struct Test {
            #[serde(deserialize_with = "nullable_default")]
            foo: HashMap<String, String>,
        }

        assert_eq!(
            serde_json::from_str::<Test>(r#"{"foo":null}"#).expect("failed to deserialize"),
            Test {
                foo: HashMap::new(),
            }
        )
    }

}
