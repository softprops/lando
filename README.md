# lando [![Build Status](https://travis-ci.org/softprops/lando.svg?branch=master)](https://travis-ci.org/softprops/lando) [![Coverage Status](https://coveralls.io/repos/github/softprops/lando/badge.svg)](https://coveralls.io/github/softprops/lando) [![crates.io](https://img.shields.io/crates/v/lando.svg)](https://crates.io/crates/lando) [![docs.rs](https://docs.rs/lando/badge.svg)](https://docs.rs/lando) [![Master API docs](https://img.shields.io/badge/docs-master-green.svg)](https://softprops.github.io/lando)

<p align="center">
  <img src="assets/logo.png" />
</p>

> [AWS lambda](https://aws.amazon.com/lambda/) [API Gateway](https://aws.amazon.com/api-gateway/) interfaces for [Rustlang](https://www.rust-lang.org) http applications.

```rust
#[macro_use] extern crate lando;

gateway!(|_, _| {
  Ok("👋 Hello, what have we here?")
});
```


## 🤔 about

Lando is a crate for **serverless** Rustlang HTTP applications.

> The rustlang ecosystem has a number of really great [HTTP server crates](https://crates.io/categories/web-programming::http-server).
A common property they all have is that they bundle servers that listen on ports that expose your application over network connections. A server which is then your reponsiblity to managing hosting, scaling, monitoring and operations for _in addition to_ your application code.

Lando is different. Lando's puts the sole focus on writing applications. It shifts the responsibility of managing servers that listen on ports that expose your application over network connections to AWS. This removes the [undifferentiated heavy lifting](https://www.cio.co.nz/article/466635/amazon_cto_stop_spending_money_undifferentiated_heavy_lifting_/) that comes along with managing servers yourself. Put more directly AWS lambda let's you run code without thinking about servers.

Lando is designed to work with the interfaces of strong existing ecosystems, both within Rust as well as the strong serverless ecosystems that exist outside Rust.

Lando's embraces the Rust community standard [http](https://crates.io/crates/http) crate as it's interface for API Gateway. Lando extends the existing work of the [crowbar](https://crates.io/crates/crowbar) crate which
provides needed lower level machinery for easily embeding a Rust application with one of lamdba's
[lowest overhead runtimes](https://medium.com/@nathan.malishev/lambda-cold-starts-language-comparison-%EF%B8%8F-a4f4b5f16a62),
Python 3.6. Lando specifically targets API Gateway triggered lambdas. Checkout crowbar for other types of [lambda triggers](https://docs.aws.amazon.com/lambda/latest/dg/invoking-lambda-function.html).

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

As a highly embeddable language, its interop story for runtimes like python's is 💖. Be mindful that lando assumes you're exposing these applications through AWS API gateway which has its own [generous pricing model](https://aws.amazon.com/api-gateway/pricing/).

## 📦  install

Add the following to your [cargo](https://doc.rust-lang.org/cargo/) project's `Cargo.toml` file.

```toml
[lib]
name = "lambda"
crate-type = ["cdylib"]

[dependencies]
lando = "0.2"
```

> 💡 The `crate-type` property informs rustc to [link](https://doc.rust-lang.org/reference/linkage.html) and produce a shared object ( `*.so` ) file allowing your rustlang application to compiled to linux native binary that can be invoked from the AWS python 3.6 [lambda runtime](https://docs.aws.amazon.com/lambda/latest/dg/current-supported-versions.html)

## 👩‍🏭 create

Lando exports a macro named `gateway!` which in turn, exports a Rust function or
closure to a cpython native binary extention making it ready for use within an AWS Lambda.

```rust
#[macro_use] extern crate lando;

gateway!(|request, _context| {
  println!("{:?}", request);
  Ok("hello lambda")
});
```

This closure accepts an `http::Request` with a [lando::Body](http://lessis.me/lando/lando/enum.Body.html). This Body type can be dereferenced as a slice of bytes if needed.

For more more in-depth details see this project's [crate documentation](http://lessis.me/lando/lando/index.html).

## 🔬 testing

Since these functions are just Rust you can test your application with the built in unit testing framework

In addition you can also integration test your functions by invoking them locally

### 🐳 Lambda CI

The [lambda CI docker project](https://github.com/lambci/docker-lambda) contains docker images that mirror the
AWS lambda runtimes. This enables you to build and test your lambda projects locally environments that match with
AWS's.

### Build

In order to invoke your function in a Lambda compatible environment you must first build it in one.

```sh
$ docker run --rm \
  -v ${PWD}:/code \
  -v ${HOME}/.cargo/registry:/root/.cargo/registry \
  -v ${HOME}/.cargo/git:/root/.cargo/git \
  -e CARGO_FLAGS="--features lando/python3-sys" \
  softprops/lambda-rust
```

This results in a native linux binary `.so` file under the`target/lambda/release` directory

### Invoke

You can use the `lambci/lambda:python3.6` docker images to invoke your lambda locally

This example provides the lambda's event though std in by piping a file, in this example a file
called `example_request.json`. Feel free to create your own mock inputs.

```sh
cat example_request.json
{
  "path": "/test/hello",
  "headers": {
    "Accept": "text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,*/*;q=0.8",
    "Accept-Encoding": "gzip, deflate, lzma, sdch, br",
    "Accept-Language": "en-US,en;q=0.8",
    "CloudFront-Forwarded-Proto": "https",
    "CloudFront-Is-Desktop-Viewer": "true",
    "CloudFront-Is-Mobile-Viewer": "false",
    "CloudFront-Is-SmartTV-Viewer": "false",
    "CloudFront-Is-Tablet-Viewer": "false",
    "CloudFront-Viewer-Country": "US",
    "Host": "wt6mne2s9k.execute-api.us-west-2.amazonaws.com",
    "Upgrade-Insecure-Requests": "1",
    "User-Agent": "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_11_6) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/52.0.2743.82 Safari/537.36 OPR/39.0.2256.48",
    "Via": "1.1 fb7cca60f0ecd82ce07790c9c5eef16c.cloudfront.net (CloudFront)",
    "X-Amz-Cf-Id": "nBsWBOrSHMgnaROZJK1wGCZ9PcRcSpq_oSXZNQwQ10OTZL4cimZo3g==",
    "X-Forwarded-For": "192.168.100.1, 192.168.1.1",
    "X-Forwarded-Port": "443",
    "X-Forwarded-Proto": "https"
  },
  "pathParameters": {
    "proxy": "hello"
  },
  "requestContext": {
    "accountId": "123456789012",
    "resourceId": "us4z18",
    "stage": "test",
    "requestId": "41b45ea3-70b5-11e6-b7bd-69b5aaebc7d9",
    "identity": {
      "cognitoIdentityPoolId": "",
      "accountId": "",
      "cognitoIdentityId": "",
      "caller": "",
      "apiKey": "",
      "sourceIp": "192.168.100.1",
      "cognitoAuthenticationType": "",
      "cognitoAuthenticationProvider": "",
      "userArn": "",
      "userAgent": "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_11_6) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/52.0.2743.82 Safari/537.36 OPR/39.0.2256.48",
      "user": ""
    },
    "resourcePath": "/{proxy+}",
    "httpMethod": "GET",
    "apiId": "wt6mne2s9k"
  },
  "resource": "/{proxy+}",
  "httpMethod": "GET",
  "queryStringParameters": {
    "name": "me"
  },
  "stageVariables": {
    "stageVarName": "stageVarValue"
  }
}

```

Invoke using `docker` providing the contexts of the mock event as stdin.

```sh
cat example_request.json | \
  docker run \
    -i -e DOCKER_LAMBDA_USE_STDIN=1 \
    --rm \
    -v \
   "$PWD/target/":/var/task lambci/lambda:python3.6 \
    liblambda.handler
```


## 🚀 deploy

In order to deploy your app you will need to build it within a runtime compatible with the
lambda python 3.6 env.

### ⚡ serverless framework

The recommended way to get started is with the [serverless framework](https://serverless.com/framework/). A [serverless framework plugin](https://github.com/softprops/serverless-rust) exists to facilitate rapid development/deployment cycles.

You can bootstrap a new deploy ready lando application by using [this serverless project template](https://github.com/softprops/serverless-lando)

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

(none)

Doug Tangren (softprops) 2018
