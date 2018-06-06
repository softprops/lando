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
          "the answer is {}", params.y + params.y
        )
      ).unwrap_or_else(
        "try again".to_string()
      )
    )
);
```

# 0.1.0

* initial release