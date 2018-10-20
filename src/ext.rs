//! conversions to and from internal gateway types and http crate types

// Std
use std::collections::HashMap;

// Third Party
use body::Body;
use http::header::CONTENT_TYPE;
use http::{Request as HttpRequest, Response as HttpResponse};
use serde::de::value::Error as SerdeError;
use serde::Deserialize;
use serde_json;
use serde_urlencoded;

// Ours
use request::{GatewayRequest, RequestContext};
use response::GatewayResponse;

/// API gateway pre-parsed http query string parameters
struct QueryStringParameters(HashMap<String, String>);

/// API gateway pre-extracted url path parameters
struct PathParameters(HashMap<String, String>);

/// API gateway configured
/// [stage variables](https://docs.aws.amazon.com/apigateway/latest/developerguide/stage-variables.html)
struct StageVariables(HashMap<String, String>);

/// Payload deserialization errors
#[derive(Debug, Fail)]
pub enum PayloadError {
    /// Returned when `application/json` bodies fail to deserialize a payload
    #[fail(display = "failed to parse payload from application/json")]
    Json(serde_json::Error),
    /// Returned when `application/x-www-form-urlencoded` bodies fail to deserialize a payload
    #[fail(display = "failed to parse payload application/x-www-form-urlencoded")]
    WwwFormUrlEncoded(SerdeError),
}

/// Extentions for `lando::Request` structs that
/// provide access to [API gateway features](https://docs.aws.amazon.com/apigateway/latest/developerguide/set-up-lambda-proxy-integrations.html#api-gateway-simple-proxy-for-lambda-input-format)
///
/// In addition, you can also access a request's body in deserialized format
/// for payloads sent in `application/x-www-form-urlencoded` or
/// `application/x-www-form-urlencoded` format
///
/// ```rust
/// #[macro_use] extern crate cpython;
/// #[macro_use] extern crate lando;
/// #[macro_use] extern crate serde_derive;
///
/// use lando::{Response, RequestExt};
///
/// #[derive(Debug,Deserialize,Default)]
/// struct Args {
///   #[serde(default)]
///   x: usize,
///   #[serde(default)]
///   y: usize
/// }
///
/// # fn main() {
/// gateway!(|request, _| {
///   let args: Args = request.payload()
///     .unwrap_or_else(|_parse_err| None)
///     .unwrap_or_default();
///   Ok(
///      Response::new(
///        format!(
///          "{} + {} = {}",
///          args.x,
///          args.y,
///          args.x + args.y
///        )
///      )
///   )
/// });
/// # }
/// ```
pub trait RequestExt {
    /// Return pre-parsed http query string parameters, parameters
    /// provided after the `?` portion of a url,
    /// associated with the API gateway request. No query parameters
    /// will yield an empty HashMap.
    fn query_string_parameters(&self) -> HashMap<String, String>;
    /// Return pre-extracted path parameters, parameter provided in url placeholders
    /// `/foo/{bar}/baz/{boom}`,
    /// associated with the API gateway request. No path parameters
    /// will yield an empty HashMap
    fn path_parameters(&self) -> HashMap<String, String>;
    /// Return [stage variables](https://docs.aws.amazon.com/apigateway/latest/developerguide/stage-variables.html)
    /// associated with the API gateway request. No stage parameters
    /// will yield an empty HashMap
    fn stage_variables(&self) -> HashMap<String, String>;
    /// Return request context data assocaited with the API gateway request
    fn request_context(&self) -> RequestContext;

    /// Return the Result of a payload parsed into a serde Deserializeable
    /// type
    ///
    /// Currently only `application/x-www-form-urlencoded`
    /// and `application/json` flavors of content type
    /// are supported
    ///
    /// A [PayloadError](enum.PayloadError.html) will be returned for undeserializable
    /// payloads. If no body is provided, `Ok(None)` will be returned.
    fn payload<D>(&self) -> Result<Option<D>, PayloadError>
    where
        for<'de> D: Deserialize<'de>;
}

impl RequestExt for HttpRequest<super::Body> {
    fn query_string_parameters(&self) -> HashMap<String, String> {
        self.extensions()
            .get::<QueryStringParameters>()
            .map(|ext| ext.0.clone())
            .unwrap_or_else(Default::default)
    }
    fn path_parameters(&self) -> HashMap<String, String> {
        self.extensions()
            .get::<PathParameters>()
            .map(|ext| ext.0.clone())
            .unwrap_or_else(Default::default)
    }
    fn stage_variables(&self) -> HashMap<String, String> {
        self.extensions()
            .get::<StageVariables>()
            .map(|ext| ext.0.clone())
            .unwrap_or_else(Default::default)
    }

    fn request_context(&self) -> RequestContext {
        self.extensions()
            .get::<RequestContext>()
            .cloned()
            .unwrap_or_else(Default::default)
    }

    fn payload<D>(&self) -> Result<Option<D>, PayloadError>
    where
        for<'de> D: Deserialize<'de>,
    {
        self.headers()
            .get(CONTENT_TYPE)
            .map(|ct| match ct.to_str() {
                Ok("application/x-www-form-urlencoded") => {
                    serde_urlencoded::from_bytes::<D>(self.body().as_ref())
                        .map_err(PayloadError::WwwFormUrlEncoded)
                        .map(Some)
                }
                Ok("application/json") => serde_json::from_slice::<D>(self.body().as_ref())
                    .map_err(PayloadError::Json)
                    .map(Some),
                _ => Ok(None),
            })
            .unwrap_or_else(|| Ok(None))
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
            headers,
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

        // build an http::Request from a lando::Request
        let mut builder = HttpRequest::builder();
        builder.method(http_method.as_str()).uri({
            format!(
                "https://{}{}",
                headers
                    .get("Host")
                    .or_else(|| headers.get("host"))
                    .unwrap_or(&String::new()),
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
                Some(b) => {
                    if is_base64_encoded {
                        // todo: document failure behavior
                        Body::from(::base64::decode(&b).unwrap_or_default())
                    } else {
                        Body::from(b.as_str())
                    }
                }
                _ => Body::from(()),
            })
            .expect("failed to build request")
    }
}

#[cfg(test)]
mod tests {
    use super::GatewayRequest;
    use http::Request as HttpRequest;
    use std::collections::HashMap;
    use RequestExt;

    #[test]
    fn requests_convert() {
        let mut headers = HashMap::new();
        headers.insert("Host".to_string(), "www.rust-lang.org".to_owned());
        let gwr: GatewayRequest = GatewayRequest {
            path: "/foo".into(),
            http_method: "GET".into(),
            headers,
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
        headers.insert("Host".to_string(), "www.rust-lang.org".to_owned());
        let mut query = HashMap::new();
        query.insert("foo".to_owned(), "bar".to_owned());
        let gwr: GatewayRequest = GatewayRequest {
            path: "/foo".into(),
            http_method: "GET".into(),
            headers,
            query_string_parameters: query.clone(),
            ..Default::default()
        };
        let actual = HttpRequest::from(gwr);
        assert_eq!(actual.query_string_parameters(), query.clone());
    }

    #[test]
    fn requests_have_form_post_parseable_payloads() {
        let mut headers = HashMap::new();
        headers.insert("Host".to_string(), "www.rust-lang.org".to_owned());
        headers.insert(
            "Content-Type".to_string(),
            "application/x-www-form-urlencoded".to_owned(),
        );
        #[derive(Deserialize, PartialEq, Debug)]
        struct Payload {
            foo: String,
            baz: usize,
        }
        let gwr: GatewayRequest = GatewayRequest {
            path: "/foo".into(),
            http_method: "GET".into(),
            headers,
            body: Some("foo=bar&baz=2".into()),
            ..Default::default()
        };
        let actual = HttpRequest::from(gwr);
        let payload: Option<Payload> = actual.payload().unwrap_or_else(|_| None);
        assert_eq!(
            payload,
            Some(Payload {
                foo: "bar".into(),
                baz: 2
            })
        )
    }

    #[test]
    fn requests_have_form_post_parseable_payloads_for_hashmaps() {
        let mut headers = HashMap::new();
        headers.insert("Host".to_string(), "www.rust-lang.org".to_owned());
        headers.insert(
            "Content-Type".to_string(),
            "application/x-www-form-urlencoded".to_owned(),
        );
        let gwr: GatewayRequest = GatewayRequest {
            path: "/foo".into(),
            http_method: "GET".into(),
            headers,
            body: Some("foo=bar&baz=2".into()),
            ..Default::default()
        };
        let actual = HttpRequest::from(gwr);
        let mut expected = HashMap::new();
        expected.insert("foo".to_string(), "bar".to_string());
        expected.insert("baz".to_string(), "2".to_string());
        let payload: Option<HashMap<String, String>> = actual.payload().unwrap_or_else(|_| None);
        assert_eq!(payload, Some(expected))
    }

    #[test]
    fn requests_have_json_parseable_payloads() {
        let mut headers = HashMap::new();
        headers.insert("Host".to_string(), "www.rust-lang.org".to_owned());
        headers.insert("Content-Type".to_string(), "application/json".to_owned());
        #[derive(Deserialize, PartialEq, Debug)]
        struct Payload {
            foo: String,
            baz: usize,
        }
        let gwr: GatewayRequest = GatewayRequest {
            path: "/foo".into(),
            http_method: "GET".into(),
            headers,
            body: Some(r#"{"foo":"bar", "baz": 2}"#.into()),
            ..Default::default()
        };
        let actual = HttpRequest::from(gwr);
        let payload: Option<Payload> = actual.payload().unwrap_or_else(|_| None);
        assert_eq!(
            payload,
            Some(Payload {
                foo: "bar".into(),
                baz: 2
            })
        )
    }
}
