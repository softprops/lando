//! Lando provides building blocks for serverless HTTP Rust applications deployable on [AWS Lambda](https://aws.amazon.com/lambda/).
//!
//! Specifically, lando exposes [API Gateway](https://aws.amazon.com/api-gateway/) proxy events
//! as standard Rust [http](https://crates.io/crates/http) types with API Gateway
//! modeled [Bodies](enum.Body.html). For convenience,
//! `lando` re-exports `http::Request` and `http::Response`.
//!
//! AWS Lambda is a ✨ **fully managed** ✨ compute service allowing you to run
//! code without thinking about servers. AWS will providing monitoring metrics and scaling
//! out of the box for you.
//!
//! Lando exports Rust functions as native CPython modules making it possible to embed
//! handlers within AWS' [Python3.6 runtime](https://docs.aws.amazon.com/lambda/latest/dg/python-programming-model.html).
//!
//! # Usage
//!
//! Add `lando` to your `Cargo.toml`
//!
//! ```toml
//! [dependencies]
//! lando = "0.1"
//! ```
//!
//! Within your application's source, use lando's macros.
//!
//! ```
//! #[macro_use]
//! extern crate lando;
//! # fn main() { }
//! ```
//!
//! Write your function using the [gateway!](macro.gateway.html) macro. See
//! it's documentation for more examples.
//!
//! ```rust
//! # #[macro_use] extern crate lando;
//! # fn main() {
//! gateway!(|_request, context| {
//!     println!("👋 cloudwatch logs, this is {}", context.function_name());
//!     // return a basic 200 response
//!     Ok(lando::Response::new(()))
//! });
//! # }
//! ```
//!
//! # Packaging functions
//!
//! Lando targets AWS Lambda's Python3.6 runtime. The
//! [gateway!](macro.gateway.html) macro does
//! the all the integration for you, but cargo still needs
//! to know what type of `lib` you are compiling. Cargo makes it easy to produce
//! compatible binaries.
//!
//! Simply add the following setting to your crate's `Cargo.toml` file.
//!
//! ```toml
//! [lib]
//! crate-type = ["cdylib"]
//! ```
//!
//! 💡 `dylib` produces dynamic library embeddable in other languages. This and other link formats are described [here](https://doc.rust-lang.org/reference/linkage.html)
//!
//! `cargo build` will then produce an AWS deploy-ready `liblambda.so` binary artifact on linux hosts.
//! Package this file in a zip file and it's now deployable as an AWS Lambda function!
//! Be sure to use the the Python 3.6 execution environment with the handler
//! configured as `lib{your_crate_name}.handler`.
//!
//! Because you're building a dynamic library, other libraries that you're dynamically linking
//! against need to also be in the Lambda execution environment. The easiest way to achive this is
//! by building in an environment similar to Lambda's. [This Docker
//! container](https://hub.docker.com/r/softprops/lambda-rust/) faithfully reproduces the AWS Lambda Python 3.6 runtime.
//!
#[cfg(test)]
#[macro_use]
extern crate pretty_assertions;
extern crate base64;
extern crate bytes;
// in addition to cpython types we use its macros in our macro
// py_module_initializer!, py_fn!
// we export and pub use those so that consumers of this
// need only have to declare one dependency
#[doc(hidden)]
pub extern crate cpython;
#[doc(hidden)]
pub use cpython::*;

extern crate crowbar;
extern crate failure;
#[macro_use]
extern crate failure_derive;
// re-export for convenience
pub extern crate http;
#[doc(hidden)]
pub extern crate mashup;
#[doc(hidden)]
pub use mashup::*;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate serde_urlencoded;

// Std
use std::error::Error as StdError;
use std::result::Result as StdResult;

// Third Party
use cpython::Python;
#[doc(hidden)]
pub use cpython::{PyObject, PyResult};
pub use crowbar::LambdaContext;

// Ours

mod body;
mod ext;
pub mod request;
mod response;
mod strmap;

pub use body::Body;
pub use ext::{PayloadError, RequestExt};
//  for benches only!
pub use request::GatewayRequest;
pub use strmap::StrMap;

/// A re-exported version of `http::Request` with a type
/// parameter for body fixed to type [lando::Body](enum.Body.html)
pub type Request = http::Request<Body>;

/// A re-exported version of the `http::Response` type
pub use http::Response;

/// Result type for gateway functions
pub type Result = StdResult<Response<Body>, Box<StdError>>;

// wrap crowbar handler in gateway handler
// which works with http crate types lifting them into apigw types
#[doc(hidden)]
pub fn handler<F, R>(
    py: Python,
    func: F,
    py_event: PyObject,
    py_context: PyObject,
) -> PyResult<PyObject>
where
    F: FnOnce(Request, LambdaContext) -> StdResult<Response<R>, Box<StdError>>,
    R: Into<Body>,
{
    crowbar::handler(
        py,
        |event, ctx| {
            let apigw = serde_json::from_value::<request::GatewayRequest>(event)?;
            func(Request::from(apigw), ctx).map(response::GatewayResponse::from)
        },
        py_event,
        py_context,
    )
}

/// A macro that exposes a Lambda function handler for AWS API gateway proxy event triggers.
///
/// Lambda functions accept two arguments (the event, a [lando::Request](type.Request.html), and a context, a
/// `LambdaContext`) and are expected to return a result containing [lando::Response](struct.Response.html). The function signature should look
/// like:
///
/// ```
/// # extern crate lando;
/// # use lando::{Request, LambdaContext, Result};
/// fn handler(
///   request: Request,
///   context: LambdaContext
/// ) -> Result {
///   // impl...
///   # Ok(lando::Response::new("docs".into()))
/// }
/// ```
///
/// To use this macro, you need the following `macro_use` declaration
///
/// ```
/// #[macro_use]
/// extern crate lando;
/// # fn main() { }
/// ```
///
/// # Examples
///
/// You can export a lambda-ready function by wrapping a closure with `gateway!`:
///
/// ```rust
/// # #[macro_use] extern crate lando;
/// # fn main() {
/// gateway!(|request, context| {
///     println!("{:?}", request);
///     Ok(lando::Response::new(()))
/// });
/// # }
/// ```
///
/// You can also the provide `gateway!` macro with a function reference
///
/// The `request` argument is just a regular `http::Request` type,
/// extendable with API gateway features, like accessing path and query string parameters, and
/// more by importing [lando::RequestExt`](trait.RequestExt.html)
///
/// The context argument is [same type](struct.LambdaContext.html) defined within the crowbar crate.
///
/// ```rust
/// # #[macro_use] extern crate lando;
/// # fn main() {
/// use lando::{LambdaContext, Request, Response, Result};
///
/// fn handler(
///   request: Request,
///   context: LambdaContext
/// ) -> Result {
///   println!("{:?}", request);
///   Ok(Response::new("👍".into()))
/// }
///
/// gateway!(handler);
/// # }
/// ```
///
/// # Export multiple lambda functions in one library
///
/// You can export multiple functions in the same module with a format similar to a `match` expression:
///
/// ```rust
/// # #[macro_use] extern crate lando;
/// # fn main() {
/// use lando::Response;
///
/// gateway! {
///     "one" => |request, context| { Ok(Response::new("1")) },
///     "two" => |request, context| { Ok(Response::new("2")) }
/// };
/// # }
/// ```
///
#[macro_export]
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
        // conventions required by cpython crate
        // https://dgrunwald.github.io/rust-cpython/doc/cpython/macro.py_module_initializer.html
        // in the future concat_indents! would be the way to make this
        // dynamic
        // see also https://www.ncameron.org/blog/untitledconcat_idents-and-macros-in-ident-position/
        // https://github.com/rust-lang/rust/issues/29599
        mashup! {
            m["modulename"] = lib env!("CARGO_PKG_NAME");
            m["py2_init"] = initlib env!("CARGO_PKG_NAME");
            m["py3_init"] = PyInit_lib env!("CARGO_PKG_NAME");
        }
        m! {
          gateway! { @module ("modulename", "py2_init", "py3_init")
                  @handlers ($($handler => $target),*) }
        }
    };
    ($($handler:expr => $target:expr,)*) => {
        gateway! { $($handler => $target),* }
    };
    ($f:expr) => {
        gateway! { "handler" => $f, }
    };
}
