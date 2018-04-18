//! Lando exposes your Rustlang functions over http using [AWS lambda](https://aws.amazon.com/lambda/)
//! by extending the [crowbar](https://crates.io/crates/crowbar) crate making
//! it possible to create type safe AWS Lambda functions in Rust that are invoked
//! by [API gateway](https://aws.amazon.com/api-gateway/) events using
//! standard [http](https://crates.io/crates/http) types.
//!
//! AWS lambda is a managed service meaning that you do not need
//! to manage servers. Instead you only focus on your application,
//! and let the platform scale your application to meet its needs.
//!
//! Lando exports Rust functions as native CPython modules making it possible to embed
//! handlers within aws' [python3.6 runtime](https://docs.aws.amazon.com/lambda/latest/dg/python-programming-model.html).
//!
//! For convenience, `lando` re-exports `http::Request` and `http::Response` types.
//!
//! # Usage
//!
//! Add both `lando` and `cpython `to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! lando = "0.1"
//! cpython = "0.1"
//! ```
//!
//! Use macros from both crates:
//!
//! ```rust,ignore
//! #[macro_use(gateway)]
//! extern crate lando;
//! // the following imports macros needed by the gateway macro
//! #[macro_use]
//! extern crate cpython;
//! ```
//!
//! And write your function using the [gateway!](macro.gateway.html) macro:
//!
//! ```rust
//! # #[macro_use(gateway)] extern crate lando;
//! # #[macro_use] extern crate cpython;
//! # fn main() {
//! gateway!(|_request, context| {
//!     println!("hi cloudwatch logs, this is {}", context.function_name());
//!     // return a basic 200 response
//!     Ok(lando::Response::default())
//! });
//! # }
//! ```
//!
//! # Packaging functions
//!
//! For your code to be usable in AWS Lambda's Python3.6 execution environment,
//! you need to compile to
//! a dynamic library with the necessary functions for CPython to run. The
//! [gateway!](macro.gateway.html) macro does
//! most of this for you, but cargo still needs to know what to do.
//!
//! You can configure cargo to build a dynamic library with the following.
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
//! > Note: cdylib exports C interface from a Rust dynamic library.
//!
//! > Link formats are described [here](https://doc.rust-lang.org/reference/linkage.html)
//!
//! `cargo build` will now produce an aws deployable `liblambda.so` binary.
//! Package this in a zip file and upload it to an AWS Lambda function.
//! Use the Python 3.6 execution environment with the handler
//! configured as `liblambda.handler`.
//!
//! Because you're building a dynamic library, other libraries that you're dynamically linking
//! against need to also be in the Lambda execution environment. The easiest way to do this is
//! building in an environment similar to Lambda's, such as Amazon Linux. You can use an [EC2
//! instance](https://aws.amazon.com/amazon-linux-ami/) or a [Docker
//! container](https://hub.docker.com/r/lambci/lambda).
//!

extern crate cpython;
extern crate crowbar;
extern crate http as rust_http;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

#[doc(hidden)]
pub use cpython::{PyObject, PyResult};
use cpython::Python;
pub use crowbar::LambdaContext;

mod request;
mod response;
mod http;
pub use http::RequestExt;

/// A re-exported version of `http::Request` with a type
/// parameter for body fixed to type `Option<String>`
pub type Request = rust_http::Request<Option<String>>;

/// A re-expored version of the `http::Response` type
pub use rust_http::Response;

/// Result type for gateway functions
pub type Result = ::std::result::Result<Response<Option<String>>, Box<std::error::Error>>;

// wrap crowbar handler in gateway handler
// which works with http crate types lifting them into apigw types
#[doc(hidden)]
pub fn handler<F>(py: Python, f: F, py_event: PyObject, py_context: PyObject) -> PyResult<PyObject>
where
    F: FnOnce(Request, LambdaContext) -> Result,
{
    crowbar::handler(
        py,
        |event, ctx| {
            let apigw = serde_json::from_value::<request::GatewayRequest>(event)?;
            f(Request::from(apigw), ctx).map(response::GatewayResponse::from)
        },
        py_event,
        py_context,
    )
}

/// Macro to wrap a Lambda function handler for api gateway events.
///
/// Lambda functions accept two arguments (the event, a `lando::Request`, and the context, a
/// `LambdaContext`) and returns a value (a serde_json `Value`). The function signature should look
/// like:
///
/// ```rust,ignore
/// fn handler(request: Request, context: LambdaContext) -> GatewayResult
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
/// You can wrap a closure with `gateway!`:
///
/// ```rust
/// # #[macro_use(gateway)] extern crate lando;
/// # #[macro_use] extern crate cpython;
/// # fn main() {
/// gateway!(|request, context| {
///     println!("{:?}", request);
///     Ok(lando::Response::default())
/// });
/// # }
/// ```
///
/// You can also define a named function:
///
/// ```rust
/// # #[macro_use(gateway)] extern crate lando;
/// # #[macro_use] extern crate cpython;
/// # fn main() {
/// use lando::{Request, Response, LambdaContext, Result};
///
/// fn handler(request: Request, context: LambdaContext) -> Result {
///     println!("{:?}", request);
///     Ok(Response::new(Some(":thumbsup:".to_owned())))
/// }
///
/// gateway!(handler);
/// # }
/// ```
///
/// # Multiple handlers
///
/// You can define multiple handlers in the same module in a way similar to `match`:
///
/// ```rust
/// # #[macro_use(gateway)] extern crate lando;
/// # #[macro_use] extern crate cpython;
/// # fn main() {
/// gateway! {
///     "one" => |request, context| { Ok(lando::Response::new(Some("1".to_string()))) },
///     "two" => |request, context| { Ok(lando::Response::new(Some("2".to_string()))) }
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
/// # #[macro_use(gateway)] extern crate lando;
/// # #[macro_use] extern crate cpython;
/// # fn main() {
/// gateway! {
///     crate (libkappa, initlibkappa, PyInit_libkappa) {
///         "handler" => |request, context| {
///            Ok(lando::Response::new(
///               Some("hi from libkappa".to_string())
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
