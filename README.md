# lando [![Build Status](https://travis-ci.org/softprops/lando.svg?branch=master)](https://travis-ci.org/softprops/lando) [![Coverage Status](https://coveralls.io/repos/github/softprops/lando/badge.svg)](https://coveralls.io/github/softprops/lando) [![crates.io](https://img.shields.io/crates/v/lando.svg)](https://crates.io/crates/lando) [![docs.rs](https://docs.rs/lando/badge.svg)](https://docs.rs/lando) [![Master API docs](https://img.shields.io/badge/docs-master-green.svg)](https://softprops.github.io/lando)

> aws lambda gateway api lambda interfaces for [Rustlang](https://www.rust-lang.org) applications

![logo](logo.png)

```rust
#[macro_use] extern crate cpython;
#[macro_use] extern crate lando;

gateway!(|_, _| {
  Ok(lando::Response::new("Hello, what have we here?"))
});
```

## 🤔 about

 🚧 👷🏿‍♀️ 👷🏽 👷‍♀️ 👷 🚧 this project is currently active under construction. expect changes.

Lando is a crate for **serverless** rustlang HTTP applications.

> The rustlang ecosystem has a number of really great [HTTP server crates](https://crates.io/categories/web-programming::http-server).
If you're interested in writing HTTP applications, you may want to evaluate them as well.
A common theme they all share is that they provide interfaces for authoring applications,
in addition to interfaces for configuring servers that listen on ports that expose your application over network connections.
A server which is then your reponsiblity to figure out how to host, scale,
monitor and manage operations and uptime for.

Lando is different. Lando's focus is solely on writing applications, freeing developers from the business and toil of the [undifferentiated heavy lifting](https://www.cio.co.nz/article/466635/amazon_cto_stop_spending_money_undifferentiated_heavy_lifting_/) that comes along with managing servers.

Lando is designed to work _with_ the interfaces of strong existing ecosystems, both within Rust as well as the strong serverless ecosystems that extend beyond Rust ( make some friends! ).

Lando's embraces the Rust community standard [http](https://crates.io/crates/http) crate. This crate was extracted from the work of a number of successful projects and was designed as a framework-agnostistic and extensible http library. Lando extends
the existing work of the [crowbar](https://crates.io/crates/crowbar) crate which
provides needed lower level machinery for easily embeding a Rust application with one of lamdba's
[lowest overhead runtimes](https://theburningmonk.com/2017/06/aws-lambda-compare-coldstart-time-with-different-languages-memory-and-code-sizes/),
python 3.6. Lando targets API Gateway triggered lambdas. Checkout crowbar for other types of [lambda triggers](https://docs.aws.amazon.com/lambda/latest/dg/invoking-lambda-function.html).

A large and mature ecosystem of tooling for AWS lambda already exists and works well,
including workflow tools like [the serverless toolkit](https://serverless.com/framework/). Because these tools are likely to already exist within organizations, the barrier of introducing Rustlang into their arsenel will be much lower.
Lando does not intend to replace these tools but instead to work well with them 👫🏾.

> 💡 You may be asking yourself, what makes Rust a good choice for Lambda?
The AWS [cost model for lambda](https://aws.amazon.com/lambda/pricing/)
is largely based on two factors: memory size and speed.
The CPU provided to applications is proportional to memory size requested.
Lambda has a pay per usage cost model billing favoring applications that are both fast and
have low memory overheads.
As a systems language, Rust is designed specifically for these kinds of needs. Rust
has a very [tiny runtime](https://www.rust-lang.org/en-US/faq.html#does-rust-have-a-runtime),
manages memory [very effciently](https://www.rust-lang.org/en-US/faq.html#is-rust-garbage-collected),
and is [extremely fast](https://www.rust-lang.org/en-US/faq.html#how-fast-is-rust).
. As a highly embeddable language, its interop story for runtimes like python's is 💖. Be mindful that lando assumes you're exposing these applications through AWS API gateway which has its own [generous pricing model](https://docs.aws.amazon.com/apigateway/latest/developerguide/limits.html).

## 📦  install

Add the following to your cargo project's `Cargo.toml` file.

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

Lando exports a macro named `gateway!` which in turn, injects a Rust function or
closure to a cpython initializer making it ready for use within an aws lambda.

```rust
#[macro_use] extern crate cpython;
#[macro_use] extern crate lando;

gateway!(|request, _context| {
  println!("{:?}", request);
  Ok(lando::Response::new(()))
});
```

This closure accepts an `http::Request` with a [lando::Body](http://lessis.me/lando/lando/enum.Body.html). This body can be dereferenced as
a slice of bytes.

For more more in-depth details see this project's [crate documentation](http://lessis.me/lando/lando/index.html).

## 🚀 deploy

In order to deploy your app you will need to build it within a runtime compatible with the
lambda python 3.6 env.

### docker

A [docker image](https://hub.docker.com/r/softprops/lambda-rust/) is provided for convenience

It's focus is on applications targetting **stable** versions of Rust.

```bash
$ docker run --rm \
        -v ${PWD}:/code \
        -v ${HOME}/.cargo/registry:/root/.cargo/registry \
        -v ${HOME}/.cargo/git:/root/.cargo/git \
        -e CARGO_FLAGS="--features lando/python3-sys" \
        softprops/lambda-rust
```

This will result in a deployable .so build artifact under a `target/lambda` directory

This file can then be zipped up for AWS lambda deployment.

### serverless framework

A [serverless framework](https://serverless.com/framework/) [plugin](https://github.com/softprops/serverless-rust) exists to facilitate rapid development/deployment cycles.

The fastest way to get started is using [this serverless project template](https://github.com/softprops/serverless-lando)

```bash
$ serverless install \
  --url https://github.com/softprops/serverless-lando \
  --name my-new-service
```

Doug Tangren (softprops) 2018
