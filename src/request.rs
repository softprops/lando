//! Request types

use std::collections::HashMap;

use serde::{Deserialize, Deserializer};

/// representation of API Gateway proxy event data
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Request {
    pub resource: String,
    pub path: String,
    pub http_method: String,
    pub headers: HashMap<String, String>,
    #[serde(deserialize_with = "nullable_map")]
    pub query_string_parameters: HashMap<String, String>,
    #[serde(deserialize_with = "nullable_map")]
    pub path_parameters: HashMap<String, String>,
    #[serde(deserialize_with = "nullable_map")]
    pub stage_variables: HashMap<String, String>,
    pub body: Option<String>,
    pub is_base64_encoded: bool,
    pub request_context: Context,
}

/// API gateway Request context
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Context {
    pub path: String,
    pub account_id: String,
    pub resource_id: String,
    pub stage: String,
    pub request_id: String,
    pub api_id: String,
}

/// deserializes (json) null values to empty hashmap
// https://github.com/serde-rs/serde/issues/1098
fn nullable_map<'de, D>(deserializer: D) -> Result<HashMap<String, String>, D::Error>
where
    D: Deserializer<'de>,
{
    let opt = Option::deserialize(deserializer)?;
    Ok(opt.unwrap_or_else(|| Default::default()))
}