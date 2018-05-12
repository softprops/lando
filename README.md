# lando [![Build Status](https://travis-ci.org/softprops/lando.svg?branch=master)](https://travis-ci.org/softprops/lando) [![Coverage Status](https://coveralls.io/repos/github/softprops/lando/badge.svg)](https://coveralls.io/github/softprops/lando) [![crates.io](https://img.shields.io/crates/v/lando.svg)](https://crates.io/crates/lando) [![docs.rs](https://docs.rs/lando/badge.svg)](https://docs.rs/lando)

> aws lambda gateway api trigger interfaces for [Rustlang](https://www.rust-lang.org) applications

## [Documentation](https://softprops.github.io/lando)


> this project is currently under construction 🚧 👷🏿‍♀️ 👷🏽 👷‍♀️ 👷 🚧

### 📦  install

Add the following to your `Cargo.toml` file.

```toml
[lib]
name = "lambda"
crate-type = ["cdylib"]

[dependencies]
lando = "0.1"
cpython = "0.1"
```

> 💡 You may be new to the `cdylib` crate type. This allows rust compile and [link](https://doc.rust-lang.org/reference/linkage.html) your application as a shared object file `*.so` allows it to be included in the AWS python 3.6 [lambda runtime](https://docs.aws.amazon.com/lambda/latest/dg/current-supported-versions.html)

### 🚀 deploy

## Roadmap

```
[x] expose typesafe interface for API gateway handlers
[ ] expose API gateway interface adapting to Rust's http crate
```

Doug Tangren (softprops) 2018
