cargo test
cargo fmt --all
cargo clippy --all --all-features
cargo +nightly udeps
cargo audit
