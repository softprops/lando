# lando [![Build Status](https://travis-ci.org/softprops/lando.svg?branch=master)](https://travis-ci.org/softprops/lando) [![Coverage Status](https://coveralls.io/repos/github/softprops/lando/badge.svg)](https://coveralls.io/github/softprops/lando) [![crates.io](https://img.shields.io/crates/v/lando.svg)](https://crates.io/crates/lando) [![docs.rs](https://docs.rs/lando/badge.svg)](https://docs.rs/lando) [![Master API docs](https://img.shields.io/badge/docs-master-green.svg)](https://softprops.github.io/lando)

<p>
<img src="https://raw.githubusercontent.com/softprops/lando/master/logo.png" width="300" />
</p>

> [AWS lambda](https://aws.amazon.com/lambda/) [API Gateway](https://aws.amazon.com/api-gateway/) interfaces for [Rustlang](https://www.rust-lang.org) http applications.

```rust
#[macro_use] extern crate cpython;
#[macro_use] extern crate lando;

gateway!(|_, _| {
  Ok(lando::Response::new("👋 Hello, what have we here?"))
});
```


## 🤔 about

Lando is a crate for **serverless** Rustlang HTTP applications.

> The rustlang ecosystem has a number of really great [HTTP server crates](https://crates.io/categories/web-programming::http-server).
A common property they all have is that they bundle servers that listen on ports that expose your application over network connections. A server which is then your reponsiblity to managing hosting, scaling, monitoring and operations for _in addition to_ your application code.

Lando is different. Lando's focus is solely on writing applications. It shifts the responsibility of hosting servers that listen on ports that exposes your application over network connections to AWS. This removes the [undifferentiated heavy lifting](https://www.cio.co.nz/article/466635/amazon_cto_stop_spending_money_undifferentiated_heavy_lifting_/) that comes along with managing servers yourself. Put more directly AWS lambda let's you run code without thinking about servers.

Lando is designed to work with, not against, the interfaces of strong existing ecosystems, both within Rust as well as the strong serverless ecosystems that exist outside Rust.

Lando's embraces the Rust community standard [http](https://crates.io/crates/http) crate as it's interface for API Gateway. Lando extends the existing work of the [crowbar](https://crates.io/crates/crowbar) crate which
provides needed lower level machinery for easily embeding a Rust application with one of lamdba's
[lowest overhead runtimes](https://theburningmonk.com/2017/06/aws-lambda-compare-coldstart-time-with-different-languages-memory-and-code-sizes/),
python 3.6. Lando specifically targets API Gateway triggered lambdas. Checkout crowbar for other types of [lambda triggers](https://docs.aws.amazon.com/lambda/latest/dg/invoking-lambda-function.html).

A *large* and *mature* ecosystem of tooling for AWS lambda already exists and works well,
including workflow tools like [the serverless toolkit](https://serverless.com/framework/). Because these tools are likely to already exist within organizations, the barrier of introducing Rustlang into their arsenel will be much lower.
Lando does not intend to replace these tools but instead to work well with them 👫🏾.

### 👍 What makes Rust a good choice for Lambda applications

The AWS [cost model for lambda](https://aws.amazon.com/lambda/pricing/)
is largely based on two factors: memory size and speed.
The CPU provided to applications is proportional to memory size requested.
Lambda has a pay per usage cost model billing favoring applications that are both fast and
have low memory overheads.

As a systems language, Rust is designed specifically for these kinds of needs. Rust
has a very [tiny runtime](https://www.rust-lang.org/en-US/faq.html#does-rust-have-a-runtime),
manages memory [very effciently](https://www.rust-lang.org/en-US/faq.html#is-rust-garbage-collected),
and is [extremely fast](https://www.rust-lang.org/en-US/faq.html#how-fast-is-rust).
.

As a highly embeddable language, its interop story for runtimes like python's is 💖. Be mindful that lando assumes you're exposing these applications through AWS API gateway which has its own [generous pricing model](https://docs.aws.amazon.com/apigateway/latest/developerguide/limits.html).

## 📦  install

Add the following to your [cargo](https://doc.rust-lang.org/cargo/) project's `Cargo.toml` file.

```toml
[lib]
name = "lambda"
crate-type = ["cdylib"]

[dependencies]
lando = "0.1"
cpython = "0.1"
```

> 💡 The `crate-type` property informs rustc to [link](https://doc.rust-lang.org/reference/linkage.html) and produce a shared object ( `*.so` ) file allowing your rustlang application to be embedded within the AWS python 3.6 [lambda runtime](https://docs.aws.amazon.com/lambda/latest/dg/current-supported-versions.html)

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

### ⚡ serverless framework

The recommended way to get started is with the [serverless framework](https://serverless.com/framework/). A [serverless framework plugin](https://github.com/softprops/serverless-rust) exists to facilitate rapid development/deployment cycles.

You can bootstramp a new deploy ready lando application by using [this serverless project template](https://github.com/softprops/serverless-lando)

```bash
$ serverless install \
  --url https://github.com/softprops/serverless-lando \
  --name my-new-service
```

### 🐳 docker

A [docker image](https://hub.docker.com/r/softprops/lambda-rust/) is provided for convenience which replicates
the AWS python3.6 env with rustlang build tooling.

It's focus is on applications targeting **stable** versions of Rust.

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

## 🏃 performance

Performance analysis for lambda applications, or any application, varies based on your usecase.
In the specific case of lando, factors include

* your use of api gateway (the HTTP loadbalancing that AWS runs that invokes your functions)
* your lambda configuration (allocation of memory and attachment to resources like VPC's)
* lambda translation layer (translating between python and rust)
* your application (that's you!)

The serverless mindset is an explicit tradeoff of control runtime for focus on application.

Your application is very capable of running in double digit milliseconds.

Lando's goal is to provide a minimally invasive translation layer between the native
python events to native rustlang http types and back

A benchmark test exists to measure that translation time
with a typical [gateway event](benches/request.json) which reports a typical
(8.65 μ (micro) second results +/- 4 μ seconds) This is not likely going to be
the bottleneck of your application


```bash
test gateway_conversion ... bench:       8,652 ns/iter (+/- 4,193)
```

### 💱 Concurrency

Consideration for concurency should be noted when approaching performance with AWS lamda.

AWS Lamda is expressly *horizontal scaled*. You scale not by spawning more threads in
a running process ( scaling up ↕️ ) but by spawning more lambdas ( scaling out ↔️ ).

A key benefit of AWS lambda is that the _platform_ handles concurrency by spawning more instances of your function *for you*. This results in some economical advantages in
they way you only pay for what you use. Bear in mind you are billed at intervals of 100 milliseconds,
so the usefulness optimizing for cost is lost once you're dipped below that point.

## 🚧 planned changes

* remove the need for explicit dependency on cpython
* remove the need for awkward dependency on lambda lib name

Doug Tangren (softprops) 2018
