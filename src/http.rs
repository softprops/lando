//! convertions to and from gateway types and http crate types

use std::collections::HashMap;

use rust_http::{Request as HttpRequest, Response as HttpResponse};
use request::GatewayRequest;
use response::GatewayResponse;
use body::Body;

pub struct QueryStringParameters(HashMap<String, String>);

pub struct PathParameters(HashMap<String, String>);

pub struct StageVariables(HashMap<String, String>);

/// Extentions for `http::Request` objects that
/// provide access to api gateway features
pub trait RequestExt {
    /// Return query string parameters associated with the request
    fn query_string_parameters(&self) -> HashMap<String, String>;
    /// Return path parameters associated with the request
    fn path_parameters(&self) -> HashMap<String, String>;
    /// Return stage variables associated with the request
    fn stage_variables(&self) -> HashMap<String, String>;
}

impl<T> RequestExt for HttpRequest<T> {
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
}

// resolve a gateway reqponse for an http::Response

impl <T> From<HttpResponse<T>> for GatewayResponse where T: Into<Body> {
    fn from(value: HttpResponse<T>) -> GatewayResponse {
         let headers = value
                .headers()
                .iter()
                .map(|(k, v)| (k.as_str().to_owned(), v.to_str().unwrap().to_owned()))
                .collect::<HashMap<String, String>>();

        GatewayResponse {
            status_code: value.status().as_u16(),
            body: match value.into_body().into() {
                Body::Empty => None,
                Body::Bytes(b) => Some(String::from_utf8_lossy(b.as_ref()).to_string())
            },
            headers: headers,
            is_base64_encoded: Default::default(), // todo: infer from Content-{Encoding,Type} headers
        }
    }
}

// resolve a http::Request from a gatway request
impl From<GatewayRequest> for HttpRequest<Body> {
    fn from(value: GatewayRequest) -> Self {
        let GatewayRequest {
            path,
            http_method,
            headers,
            query_string_parameters,
            path_parameters,
            stage_variables,
            body,
            is_base64_encoded,
            request_context, // todo: expose this as an ext
        } = value;

        // build an http::Result from a lando::Request
        let mut builder = HttpRequest::builder();
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

        // todo: handle base64 decoding if needed
        if is_base64_encoded {

        }
        builder.body(match body {
            Some(b) => if is_base64_encoded {
                Body::from(::base64::decode(&b).unwrap()) // :|
            } else {
                Body::from(b.as_str())
            },
            _ => Body::from(())
        }).expect("failed to build request")
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use rust_http::Request as HttpRequest;
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
        let expected = HttpRequest::get("https://www.rust-lang.org/foo")
            .body(())
            .unwrap();
        let actual = HttpRequest::from(gwr);
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
        let actual = HttpRequest::from(gwr);
        assert_eq!(actual.query_string_parameters(), query.clone());
    }
}
