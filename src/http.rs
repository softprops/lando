//! convertions to and from gateway types and http crate types

use std::collections::HashMap;

use rust_http::{Request, Response};
use request::GatewayRequest;
use response::GatewayResponse;

pub struct QueryStringParameters(HashMap<String, String>);

pub struct PathParameters(HashMap<String, String>);

pub struct StageVariables(HashMap<String, String>);

pub struct Base64Encoded(bool);

/// Extentions for `http::Request` objects that
/// provide access to api gateway features
pub trait RequestExt {
    /// Return query string parameters associated with the request
    fn query_string_parameters(&self) -> HashMap<String, String>;
    /// Return path parameters associated with the request
    fn path_parameters(&self) -> HashMap<String, String>;
    /// Return stage variables associated with the request
    fn stage_variables(&self) -> HashMap<String, String>;
    /// Return a boolean indicator that this request's body is or is not base64 encode
    fn is_base64_encoded(&self) -> bool;
}

impl<T> RequestExt for Request<T> {
    fn query_string_parameters(&self) -> HashMap<String, String> {
        self.extensions()
            .get::<QueryStringParameters>()
            .map(|ext| ext.0.clone())
            .unwrap_or(Default::default())
    }
    fn path_parameters(&self) -> HashMap<String, String> {
        self.extensions()
            .get::<PathParameters>()
            .map(|ext| ext.0.clone())
            .unwrap_or(Default::default())
    }
    fn stage_variables(&self) -> HashMap<String, String> {
        self.extensions()
            .get::<StageVariables>()
            .map(|ext| ext.0.clone())
            .unwrap_or(Default::default())
    }

    fn is_base64_encoded(&self) -> bool {
        self.extensions()
            .get::<Base64Encoded>()
            .map(|ext| ext.0)
            .unwrap_or(false)
    }
}

// resolve a gateway reqponse for an http::Response

impl From<Response<Option<String>>> for GatewayResponse {
    fn from(value: Response<Option<String>>) -> GatewayResponse {
        GatewayResponse::builder()
            .status_code(value.status().as_u16())
            .body(value.body().clone())
            .headers(
                value
                    .headers()
                    .iter()
                    .map(|(k, v)| (k.as_str().to_owned(), v.to_str().unwrap().to_owned()))
                    .collect::<Vec<(String, String)>>(),
            )
            .build()
    }
}

// resolve a http::Request from a gatway request
impl From<GatewayRequest> for Request<Option<String>> {
    fn from(value: GatewayRequest) -> Self {
        let GatewayRequest {
            resource,
            path,
            http_method,
            headers,
            query_string_parameters,
            path_parameters,
            stage_variables,
            body,
            is_base64_encoded,
            request_context,
        } = value;

        // build an http::Result from a lando::Request
        let mut builder = Request::builder();
        builder.method(http_method.as_str()).uri({
            format!(
                "https://{}{}",
                headers.get("Host").unwrap_or(&"???".to_owned()),
                path
            )
        });
        for (k, v) in headers {
            builder.header(k.as_str(), v.as_str());
        }

        builder.extension(QueryStringParameters(query_string_parameters));
        builder.extension(PathParameters(path_parameters));
        builder.extension(StageVariables(stage_variables));
        builder.body(body).expect("failed to build request")
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use rust_http::Request;
    use RequestExt;
    use super::GatewayRequest;

    #[test]
    fn requests_convert() {
        let mut headers = HashMap::new();
        headers.insert("Host".to_owned(), "www.rust-lang.org".to_owned());
        let gwr: GatewayRequest = GatewayRequest {
            path: "/foo".into(),
            http_method: "GET".into(),
            headers: headers,
            ..Default::default()
        };
        let expected = Request::get("https://www.rust-lang.org/foo")
            .body(())
            .unwrap();
        let actual = Request::from(gwr);
        assert_eq!(expected.uri(), actual.uri());
        assert_eq!(expected.method(), actual.method());
    }

    #[test]
    fn requests_have_query_string_ext() {
        let mut headers = HashMap::new();
        headers.insert("Host".to_owned(), "www.rust-lang.org".to_owned());
        let mut query = HashMap::new();
        query.insert("foo".to_owned(), "bar".to_owned());
        let gwr: GatewayRequest = GatewayRequest {
            path: "/foo".into(),
            http_method: "GET".into(),
            headers: headers,
            query_string_parameters: query.clone(),
            ..Default::default()
        };
        let expected = Request::get("https://www.rust-lang.org/foo")
            .body(())
            .unwrap();
        let actual = Request::from(gwr);
        assert_eq!(actual.query_string_parameters(), query.clone());
    }
}
