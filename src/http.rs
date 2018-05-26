//! convertions to and from internal gateway types and http crate types

// Std
use std::collections::HashMap;

// Third Party
use body::Body;
use request::{GatewayRequest, RequestContext};
use response::GatewayResponse;
use rust_http::{Request as HttpRequest, Response as HttpResponse};

/// API gateway pre-parsed http query string parameters
struct QueryStringParameters(HashMap<String, String>);

/// API gateway pre-extracted url path parameters
struct PathParameters(HashMap<String, String>);

/// API gateway configured
/// [stage variables](https://docs.aws.amazon.com/apigateway/latest/developerguide/stage-variables.html)
struct StageVariables(HashMap<String, String>);

/// Extentions for `lando::Request` objects that
/// provide access to API gateway features
pub trait RequestExt {
    /// Return pre-parsed http query string parameters
    /// associated with the API gateway request
    fn query_string_parameters(&self) -> HashMap<String, String>;
    /// Return pre-extracted path parameters
    /// associated with the API gateway request
    fn path_parameters(&self) -> HashMap<String, String>;
    /// Return [stage variables](https://docs.aws.amazon.com/apigateway/latest/developerguide/stage-variables.html)
    /// associated with the API gateway request
    fn stage_variables(&self) -> HashMap<String, String>;
    /// Return request context assocaited with the API gateway request
    fn request_context(&self) -> RequestContext;
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

    fn request_context(&self) -> RequestContext {
        self.extensions()
            .get::<RequestContext>()
            .map(|ext| ext.clone())
            .unwrap_or(Default::default())
    }
}

// resolve a gateway reqponse for an http::Response

impl<T> From<HttpResponse<T>> for GatewayResponse
where
    T: Into<Body>,
{
    fn from(value: HttpResponse<T>) -> GatewayResponse {
        let headers = value
            .headers()
            .into_iter()
            .map(|(k, v)| {
                (
                    k.as_str().to_owned(),
                    v.to_str().unwrap_or_default().to_owned(),
                )
            })
            .collect::<HashMap<String, String>>();

        GatewayResponse {
            status_code: value.status().as_u16(),
            body: match value.into_body().into() {
                Body::Empty => None,
                Body::Bytes(b) => Some(String::from_utf8_lossy(b.as_ref()).to_string()),
            },
            headers: headers,
            is_base64_encoded: Default::default(), // todo: infer from Content-{Encoding,Type} headers
        }
    }
}

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
            request_context,
        } = value;

        // build an http::Result from a lando::Request
        let mut builder = HttpRequest::builder();
        builder.method(http_method.as_str()).uri({
            format!(
                "https://{}{}",
                headers.get("Host").unwrap_or(&String::new()),
                path
            )
        });
        for (k, v) in headers {
            builder.header(k.as_str(), v.as_str());
        }

        builder.extension(QueryStringParameters(query_string_parameters));
        builder.extension(PathParameters(path_parameters));
        builder.extension(StageVariables(stage_variables));
        builder.extension(request_context);

        builder
            .body(match body {
                Some(b) => if is_base64_encoded {
                    // todo: document failure behavior
                    Body::from(::base64::decode(&b).unwrap_or_default())
                } else {
                    Body::from(b.as_str())
                },
                _ => Body::from(()),
            })
            .expect("failed to build request")
    }
}

#[cfg(test)]
mod tests {
    use super::GatewayRequest;
    use RequestExt;
    use rust_http::Request as HttpRequest;
    use std::collections::HashMap;

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
