# 0.2 (unreleased)

* ease dependency declartions by re-exporting `http` crate

before

... in your `Cargo.toml` file

```toml
[dependencies]
lando = "0.1"
http = "0.1" # need to depend on http crate explicitly
```

... in your `src/lib.rs`

```rust
#[macro_use]
extern crate lando;
// need to extern http crate explicitly
extern crate http;

use http::{Method, StatusCode};
```

after

... in your `Cargo.toml`

```toml
[dependencies]
lando = "0.1" # no longer need to add a dependency on http explicitly
```

... in your `src/lib.rs`

```rust
#[macro_use]
extern crate lando;

// consume http re-export from lando crate
use lando::http::{Method, StatusCode};
```

* remove the need to explicitly declare cpython as a dependency, both as a depenency and macro_use

before

... in your `Cargo.toml` file

```toml
[dependencies]
lando = "0.1"
cpython = "0.1" # need to depend on cpython crate explicitly for its macros
```

... in your `src/lib.rs` file

```rust
#[macro_use]
extern crate lando;
// needed because lando's macros used cpython macros,
// an impl detail
#[macro_use]
extern crate cpython;
```

after

... in your `Cargo.toml` file

```toml
[dependencies]
lando = "0.1" # no longer need to declar cpython as an explicit dependency
```

... in your `src/lib.rs` file

```rust
#[macro_use]
extern crate lando; // impl details are hidden
```

* reduced exposed surface area of internal interfaces

`lando::GatewayRequest` still is still public (for benchmarking) but its fields are not

* reduced cost of transformations between lambda proxy types and rust native `http` crate types

Replaced many owned `String` types with `Cow`. Now deserializing and serializing request headers directly to `http::HeaderMap`

* `RequestExt` methods now return `strmap::StrMap` type instead of `HashMap<String, String>`
* introducing `IntoResponse` trait

Handlers can now return anything that implementds IntoResponse. As a result `lando::Result` type now takes a type parameter for those types. Implementations for `lando::Response` and types that implement `Into<Body>` are provided

* introducing `#[lando]` proc macro.

You can no attribute a bare function with `#[lando]` to have it exported

```rust
#[lando]
fn handler<'a>(_: Request, _: LambdaContext) -> Result<&'a str> {
  Ok("hello lando")
}
```

You can no annotate

# 0.1.1

* bug fix - support for reading host from "host" (lowercase) in addition to "Host"
* feature - add support for "application/x-www-form-urlencoded" and "application/json"
  parsed request bodies with `lando::RequestExt#payload()`

```rust
#[macro_use] extern crate cpython;
#[macro_use] extern crate lando;
#[macro_use] extern crate serde_deserialize;

use lando::{Response, RequestEx};

#[derive(Deserialize, Debug)]
struct Params {
  x: usize,
  y: usize
}

gateway!(
  |req, _| => Ok(
    Response::new(
      req.payload::<Params>().unwrap_or_else(|_| None).map(
        |params| format!(
          "the answer is {}", params.x + params.y
        )
      ).unwrap_or_else(
        || "try again".to_string()
      )
    )
  )
);
```

# 0.1.0

* initial release