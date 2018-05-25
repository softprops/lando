# lando [![Build Status](https://travis-ci.org/softprops/lando.svg?branch=master)](https://travis-ci.org/softprops/lando) [![Coverage Status](https://coveralls.io/repos/github/softprops/lando/badge.svg)](https://coveralls.io/github/softprops/lando) [![crates.io](https://img.shields.io/crates/v/lando.svg)](https://crates.io/crates/lando) [![docs.rs](https://docs.rs/lando/badge.svg)](https://docs.rs/lando) [![Master API docs](https://img.shields.io/badge/docs-master-green.svg)](https://softprops.github.io/lando)

> aws lambda gateway api trigger interfaces for [Rustlang](https://www.rust-lang.org) applications

```rust
#[macro_use(gateway)] extern crate lando;
#[macro_use] extern crate cpython;

gateway!(|request, _| {
  Ok(lando::Response::new("Hello, what have we here?))
);
```


>  🚧 👷🏿‍♀️ 👷🏽 👷‍♀️ 👷 🚧 this project is currently under construction

## 🤔 about

Lando is a crate for **serverless** HTTP applications.

> A number of really great HTTP server crates exist within the [Rust ecosystem](https://crates.io/categories/web-programming::http-server).
You should check them out!
A common theme they all share is in providing interfaces for authoring applications,
in addition to interfaces for configuring a server that listens on a port that hosts your application.
A server which is then your reponsiblity to figure out how to host, scale,
monitor and manage operations and uptime for.

Lando is different. Lando's focus is solely on applications, freeing you from the toil of [undifferentiated heavy lifting](https://www.cio.co.nz/article/466635/amazon_cto_stop_spending_money_undifferentiated_heavy_lifting_/).

Lando is designed to work the interfaces of strong existing ecosystems, both with Rust as well as
the strong serverless ecosystems that extend beyond Rust ( make some friends! ).

Lando's interfaces are based the Rust community standard [http](https://crates.io/crates/http) crate, designed as a framework-agnostistic and extensible http library, and extends
the existing work of the [crowbar](https://crates.io/crates/crowbar) crate which
provides needed lower level machinery for easily embeding a rust application with one of lamdba's
[lowest overhead runtimes](https://theburningmonk.com/2017/06/aws-lambda-compare-coldstart-time-with-different-languages-memory-and-code-sizes/),
python 3.6.

A large and mature ecosystem of tooling for AWS lambda already exists and works well,
including flowflow tools like [the serverless toolkit](https://serverless.com/framework/).
Lando does not intend to replace these but instead to work well with them 👫🏾.

> 💡 You may be asking yourself, what makes Rust a suitable choice for Lambda?
The AWS cost model for lambda is based on two factors: size and speed.
Lambda has a pay per usage cost model billing based on function size and execution time.
As a systems language, Rust is designed specifically for these kinds of needs. As a highly embeddable
language, its interop story for runtimes like python's is 💖.

## 📦  install

Add the following to your `Cargo.toml` file.

```toml
[lib]
name = "lambda"
crate-type = ["cdylib"]

[dependencies]
lando = "0.1"
cpython = "0.1"
```

> 💡 You may be new to the `cdylib` and `crate-type` lib attributes. This informs rustc to [link](https://doc.rust-lang.org/reference/linkage.html) and produce a shared object ( `*.so` ) file allowing your rustlang application to be embedded within the AWS python 3.6 [lambda runtime](https://docs.aws.amazon.com/lambda/latest/dg/current-supported-versions.html)

## 👩‍🏭 create

Lando exports a macro named `gateway!` which in turn, exports a Rust function or
closure to a cpython initializer for use within an aws lambda.

```rust
#[macro_use(gateway)] extern crate lando;
#[macro_use] extern crate cpython;

gateway!(|request, _context| {
  println!("{:?}", request);
  Ok(lando::Response::new(()))
);
```

## 🚀 deploy


Doug Tangren (softprops) 2018
