#!/bin/bash
# test, build and pack a rust lambda library

set -euo pipefail
mkdir -p target/lambda
export CARGO_TARGET_DIR=$PWD/target/lambda
(
    if [[ $# -gt 0 ]]; then
        yum install -y "$@"
    fi
    . $HOME/.cargo/env
    cargo test ${CARGO_FLAGS:-} --release
    cargo build ${CARGO_FLAGS:-} --release
) 1>&2
cd "$CARGO_TARGET_DIR"/release
(
    for file in $(
      find -maxdepth 1 -executable -type f \( -name "*.*" -or -name "*script*" -or -regex ".*-[a-f0-9]{16}" \)
    ); do
        strip "$file"
        zip lambda.zip "$file"
    done
) 1>&2