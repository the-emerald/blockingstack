# Companion Program for W14-SP
This is a thread-safe fixed-size queue that blocks on popping when empty and pushing when full.

Written in Rust. [Install it here!](https://www.rust-lang.org/tools/install)
## Build
`cargo build --release`

## Run tests
`cargo test --release -- --test-threads=1`