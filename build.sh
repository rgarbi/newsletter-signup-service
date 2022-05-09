#!/bin/zsh

cargo build
cargo test
cargo clippy
cargo fmt
cargo sqlx prepare -- --lib

docker run --security-opt seccomp=unconfined -v "${PWD}:/volume" xd009642/tarpaulin
