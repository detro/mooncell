#!/bin/bash

set -e -u -o pipefail

cd $GITHUB_WORKSPACE

echo "*** CARGO BUILD ***"
cargo build

echo "*** CARGO TEST ***"
cargo test
