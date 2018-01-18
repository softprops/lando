//! gateway extends the crowbar crate makes it easy to write AWS Lambda functions in Rust that are invoked
//! on api gateway events. It wraps native Rust functions
//! into CPython modules that handle converting Python objects into Rust objects and back again.
//!
//! # Usage
//!
//! Add both gateway and cpython to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! gateway = "0.1"
//! cpython = "0.1"
//! ```
//!
//! Use macros from both crates:
//!
//! ```rust,ignore
//! #[macro_use(gateway)]
//! extern crate gateway;
//! #[macro_use]
//! extern crate cpython;
//! ```
//!
//! And write your function using the `gateway!` macro:
//!
//! ```rust
//! # #[macro_use(gateway)] extern crate gateway;
//! # #[macro_use] extern crate cpython;
//! # fn main() {
//! gateway!(|_request, context| {
//!     println!("hi cloudwatch logs, this is {}", context.function_name());
//!     // return the event without doing anything with it
//!     Ok(gateway::Response { ..Default::default() })
//! });
//! # }
//! ```
//!
//! # Building Lambda functions
//!
//! For your code to be usable in AWS Lambda's Python execution environment, you need to compile to
//! a dynamic library with the necessary functions for CPython to run. The `lambda!` macro does
//! most of this for you, but cargo still needs to know what to do.
//!
//! You can configure cargo to build a dynamic library with the following. If you're using the
//! `lambda!` macro as above, you need to use `lambda` for the library name (see the documentation
//! for `lambda!` if you want to use something else).
//!
//! ```toml
//! [lib]
//! name = "lambda"
//! crate-type = ["cdylib"]
//! ```
//!
//! `cargo build` will now build a `liblambda.so`. Put this in a zip file and upload it to an AWS
//! Lambda function. Use the Python 3.6 execution environment with the handler configured as
//! `liblambda.handler`.
//!
//! Because you're building a dynamic library, other libraries that you're dynamically linking
//! against need to also be in the Lambda execution environment. The easiest way to do this is
//! building in an environment similar to Lambda's, such as Amazon Linux. You can use an [EC2
//! instance](https://aws.amazon.com/amazon-linux-ami/) or a [Docker
//! container](https://hub.docker.com/_/amazonlinux/).
//!
//! The `builder` directory of the [crowbar git repo](https://github.com/ilianaw/rust-crowbar)
//! contains a `Dockerfile` with Rust set up and a build script to dump a zip file containing a
//! stripped shared library to stdout. Documentation for using that is available at
//! [ilianaw/crowbar-builder on Docker Hub](https://hub.docker.com/r/ilianaw/crowbar-builder/).
//!

extern crate crowbar;
extern crate cpython;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

use std::ops::Not;
use std::collections::HashMap;

use serde::{Deserialize, Deserializer};
pub use crowbar::LambdaContext;
#[doc(hidden)]
pub use cpython::{PyObject, PyResult};
use cpython::Python;

#[derive(Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct Request {
    pub path: String,
    #[serde(rename = "httpMethod")]
    pub method: String,
    pub headers: HashMap<String, String>,
    #[serde(rename = "queryStringParameters", deserialize_with = "nullable_map")]
    pub query_params: HashMap<String, String>,
    #[serde(rename = "pathParameters", deserialize_with = "nullable_map")]
    pub path_params: HashMap<String, String>,
    pub body: Option<String>,
    pub is_base64_encoded: bool,
    pub request_context: Context,
}

#[derive(Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct Context {
    pub path: String,
    pub account_id: String,
    pub resource_id: String,
    pub stage: String,
    pub request_id: String,
    pub api_id: String,
}

#[derive(Serialize, Debug, Default)]
pub struct Response {
    #[serde(rename = "statusCode")]
    pub status_code: u16,
    pub headers: HashMap<String, String>,
    pub body: String,
    #[serde(rename = "isBase64Encoded", skip_serializing_if = "Not::not")]
    pub is_bas64_encoded: bool,
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

/// Result type for api gateway requests
pub type GatewayResult = Result<Response, Box<std::error::Error>>;

// wrap crowbar handler in gateway handler
#[doc(hidden)]
pub fn handler<F>(py: Python, f: F, py_event: PyObject, py_context: PyObject) -> PyResult<PyObject>
where
    F: FnOnce(Request, LambdaContext) -> GatewayResult,
{
    crowbar::handler(
        py,
        |event, ctx| f(serde_json::from_value::<Request>(event)?, ctx),
        py_event,
        py_context,
    )
}

#[macro_export]
/// Macro to wrap a Lambda function handler for api gateway events.
///
/// Lambda functions accept two arguments (the event, a `gateway::Request`, and the context, a
/// `LambdaContext`) and returns a value (a serde_json `Value`). The function signature should look
/// like:
///
/// ```rust,ignore
/// fn handler(event: Request, context: LambdaContext) -> GatewayResult
/// ```
///
/// To use this macro, you need to `macro_use` both crowbar *and* cpython, because crowbar
/// references multiple cpython macros.
///
/// ```rust,ignore
/// #[macro_use(gateway)]
/// extern crate gateway;
/// #[macro_use]
/// extern crate cpython;
/// ```
///
/// # Examples
///
/// You can wrap a closure with `gateway!`:
///
/// ```rust
/// # #[macro_use(gateway)] extern crate gateway;
/// # #[macro_use] extern crate cpython;
/// # fn main() {
/// gateway!(|request, context| {
///     println!("{:?}", request);
///     Ok(gateway::Response { status_code: 200, ..Default::default() })
/// });
/// # }
/// ```
///
/// You can also define a named function:
///
/// ```rust
/// # #[macro_use(gateway)] extern crate gateway;
/// # #[macro_use] extern crate cpython;
/// # fn main() {
/// use gateway::{Request, Response, LambdaContext, GatewayResult};
///
/// fn my_handler(event: Request, context: LambdaContext) -> GatewayResult {
///     println!("{:?}", event);
///      Ok(Response { status_code: 200, ..Default::default() })
/// }
///
/// gateway!(my_handler);
/// # }
/// ```
///
/// # Multiple handlers
///
/// You can define multiple handlers in the same module in a way similar to `match`:
///
/// ```rust
/// # #[macro_use(gateway)] extern crate gateway;
/// # #[macro_use] extern crate cpython;
/// # fn main() {
/// gateway! {
///     "one" => |event, context| { Ok(gateway::Response { ..Default::default() }) },
///     "two" => |event, context| { Ok(gateway::Response { ..Default::default() }) },
/// };
/// # }
/// ```
///
/// # Changing the dynamic library name
///
/// If you need to change the name of the built dynamic library, you first need to change the
/// `[lib]` section in Cargo.toml:
///
/// ```toml
/// [lib]
/// name = "kappa"
/// crate-type = ["cdylib"]
/// ```
///
/// You then also need to change the names of the library symbols, which you can do by extending
/// upon the multiple handler version of `gateway!`:
///
/// ```rust
/// # #[macro_use(gateway)] extern crate gateway;
/// # #[macro_use] extern crate cpython;
/// # fn main() {
/// gateway! {
///     crate (libkappa, initlibkappa, PyInit_libkappa) {
///         "handler" => |event, context| { Ok(gateway::Response { body: "hi from libkappa".into(), ..Default::default() }) }
///     }
/// };
/// # }
/// ```
macro_rules! gateway {
    (@module ($module:ident, $py2:ident, $py3:ident)
     @handlers ($($handler:expr => $target:expr),*)) => {
        py_module_initializer!($module, $py2, $py3, |py, m| {
            $(
                m.add(py, $handler, py_fn!(
                    py,
                    x(
                        event: $crate::PyObject,
                        context: $crate::PyObject
                    ) -> $crate::PyResult<$crate::PyObject> {
                        $crate::handler(py, $target, event, context)
                    }
                ))?;
            )*
            Ok(())
        });
    };

    (crate $module:tt { $($handler:expr => $target:expr),* }) => {
        gateway! { @module $module @handlers ($($handler => $target),*) }
    };

    (crate $module:tt { $($handler:expr => $target:expr,)* }) => {
        gateway! { @module $module @handlers ($($handler => $target),*) }
    };

    ($($handler:expr => $target:expr),*) => {
        gateway! { @module (liblambda, initliblambda, PyInit_liblambda)
                  @handlers ($($handler => $target),*) }
    };

    ($($handler:expr => $target:expr,)*) => {
        gateway! { $($handler => $target),* }
    };

    ($f:expr) => {
        gateway! { "handler" => $f, }
    };
}
