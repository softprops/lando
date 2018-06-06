//! Lando provides building blocks for serverless HTTP Rust applications deployable on [AWS lambda](https://aws.amazon.com/lambda/).
//!
//! Lando extends the [crowbar](https://crates.io/crates/crowbar) crate with
//! type-safe interfaces exposing [API gateway](https://aws.amazon.com/api-gateway/) proxy events
//! as standard Rust [http](https://crates.io/crates/http) types. For convenience,
//! `lando` re-exports `http::Request` and `http::Response` types.
//!
//! AWS lambda is a ✨ **fully managed** ✨ compute service meaning that you do not need
//! to own or operate any of the servers your application will run on, freeing
//! you up to **focus on your application**. You can consider Lambda AWS's Function-As-A-Service offering.
//!
//! Lando exports Rust functions as native CPython modules making it possible to embed
//! handlers within AWS' [python3.6 runtime](https://docs.aws.amazon.com/lambda/latest/dg/python-programming-model.html).
//!
//! # Usage
//!
//! Add both `lando` and `cpython` as dependencies to your `Cargo.toml`
//!
//! ```toml
//! [dependencies]
//! cpython = "0.1"
//! lando = "0.1"
//! ```
//!
//! Within your libraries source, use the macros from both crates
//!
//! ```rust,ignore
//! // the following imports macros needed by the gateway macro
//! #[macro_use]
//! extern crate cpython;
//! #[macro_use(gateway)]
//! extern crate lando;
//! ```
//!
//! And write your function using the [gateway!](macro.gateway.html) macro
//!
//! ```rust
//! # #[macro_use] extern crate cpython;
//! # #[macro_use(gateway)] extern crate lando;
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
//! Lando targets AWS Lambda's Python3.6 runtime. For your code to be usable
//! in this execution environment, you need to compile your application as
//! a dynamic library allowing it to be embedded within CPython. The
//! [gateway!](macro.gateway.html) macro does
//! the all the integration for you, but cargo still needs
//! to know what type of lib you are compiling.
//!
//! You can configure cargo to build a dynamic library with the following toml.
//! If you're using the
//! `gateway!` macro as above, you need to use `lambda` for the library name
//! (see the documentation
//! for [gateway!](macro.gateway.html) if you want to use something else).
//!
//! ```toml
//! [lib]
//! name = "lambda"
//! crate-type = ["cdylib"]
//! ```
//!
//! > 💡 `dylib` produces dynamic library embeddable in other languages. This and other link formats are described [here](https://doc.rust-lang.org/reference/linkage.html)
//!
//! `cargo build` will then produce an AWS-deployable `liblambda.so` binary artifact.
//! Package this file in a zip file and its now deployable as an AWS Lambda function!
//! Be sure to use the the Python 3.6 execution environment with the handler
//! configured as `liblambda.handler`.
//!
//! Because you're building a dynamic library, other libraries that you're dynamically linking
//! against need to also be in the Lambda execution environment. The easiest way to achive this is
//! by building in an environment similar to Lambda's, like [this Docker
//! container](https://hub.docker.com/r/softprops/lambda-rust/).
//!
#[cfg(test)]
#[macro_use]
extern crate pretty_assertions;
extern crate base64;
extern crate bytes;
extern crate cpython;
extern crate crowbar;
extern crate http as rust_http;
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

mod body;
mod http;
mod request;
mod response;

pub use body::Body;
pub use http::RequestExt;
pub use request::GatewayRequest;

/// A re-exported version of `http::Request` with a type
/// parameter for body fixed to type [lando::Body](enum.Body.html)
pub type Request = rust_http::Request<Body>;

/// A re-exported version of the `http::Response` type
pub use rust_http::Response;

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

/// Macro that exposes a Lambda function handler for AWS API gateway proxy event triggers.
///
/// Lambda functions accept two arguments (the event, a [lando::Request](type.Request.html), and a context, a
/// `LambdaContext`) and are expected to return a result containing [lando::Response](struct.Response.html). The function signature should look
/// like:
///
/// ```rust,ignore
/// fn handler(request: Request, context: LambdaContext) -> Result
/// ```
///
/// To use this macro, you need to `macro_use` both crowbar *and* cpython, because crowbar
/// references multiple cpython macros.
///
/// ```rust,ignore
/// #[macro_use(gateway)]
/// extern crate lando;
/// #[macro_use]
/// extern crate cpython;
/// ```
///
/// # Examples
///
/// You can export a lambda-ready function by wrapping a closure with `gateway!`:
///
/// ```rust
/// # #[macro_use(gateway)] extern crate lando;
/// # #[macro_use] extern crate cpython;
/// # fn main() {
/// gateway!(|request, context| {
///     println!("{:?}", request);
///     Ok(lando::Response::new(()))
/// });
/// # }
/// ```
///
/// You can also the provide `gateway!` macro with a named function
///
/// The request argument is just a regular `http::Request` type but you can
/// extend with API gateway features, like path and query string parameters, and
/// more by importing [lando::RequestExt`](trait.RequestExt.html)
///
/// The context argument is [same type](struct.LambdaContext.html) used within the crowbar crate
///
/// ```rust
/// # #[macro_use(gateway)] extern crate lando;
/// # #[macro_use] extern crate cpython;
/// # fn main() {
/// use lando::{LambdaContext, Request, Response, Result, Body};
///
/// fn handler(request: Request, context: LambdaContext) -> Result {
///     println!("{:?}", request);
///     Ok(Response::new("👍".into()))
/// }
///
/// gateway!(handler);
/// # }
/// ```
///
/// # Multiple functions
///
/// You can export multiple functions in the same module with a format similar to a `match` expression:
///
/// ```rust
/// # #[macro_use(gateway)] extern crate lando;
/// # #[macro_use] extern crate cpython;
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
/// # Changing the dynamic library name
///
/// Be default, lando assumes a library named "lambda", If you need to change the
/// name of the resulting dynamic library that gets built,
///  you first need to change the `[lib]` section in your Cargo.toml file
///
/// ```toml
/// [lib]
/// name = "solo"
/// crate-type = ["cdylib"]
/// ```
///
/// You then also need to change the names of the library identifiers, expected by
/// the [cpython crate](https://dgrunwald.github.io/rust-cpython/doc/cpython/macro.py_module_initializer.html),
/// by using the following `gateway!` format. This pattern may no longer needed
/// the std library's [concat_idents!](https://doc.rust-lang.org/std/macro.concat_idents.html)
/// macro is stablized.
///
/// ```rust
/// # #[macro_use(gateway)] extern crate lando;
/// # #[macro_use] extern crate cpython;
/// # fn main() {
/// gateway! {
///     crate (libsolo, initlibsolo, PyInit_libsolo) {
///         "handler" => |request, context| {
///            Ok(lando::Response::new(
///               "hello from libsolo"
///            ))
///         }
///     }
/// };
/// # }
/// ```
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
        gateway! { @module (liblambda,
                            initliblambda,
                            PyInit_liblambda)
                  @handlers ($($handler => $target),*) }
    };
    ($($handler:expr => $target:expr,)*) => {
        gateway! { $($handler => $target),* }
    };
    ($f:expr) => {
        gateway! { "handler" => $f, }
    };
}
