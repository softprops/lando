# lambda rust docker builder

## about

This image extends [lambda ci python 3.6](https://github.com/lambci/docker-lambda#documentation) docker image which is a reproduction of the actual python 3.6 lambda runtime environment
and installs [rustup](https://rustup.rs/) and the stable rust toolchain.

## install

Tags for this docker follow the convention `softprops/lambda-rust:{version}-rust-{rust-version}'
Where rust-version is a stable version of rust.

## usage

The default docker command will build a release version your rust application under `target/lambda/` to
isolate the lambda specific build artifacts from your localhost build artifacts.

You will want to volume mount `/code` to the directory containing your cargo project.

You can pass additional flags to cargo by setting the `CARGO_FLAGS` docker env variable

A typical run might look like the following

```bash
docker run --rm \
		-v ${PWD}:/code \
		-v ${HOME}/.cargo/registry:/root/.cargo/registry \
		-v ${HOME}/.cargo/git:/root/.cargo/git \
		-e CARGO_FLAGS="--features python3-sys" \
		softprops/lambda-rust:{tag}
```
