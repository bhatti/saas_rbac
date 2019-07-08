./db.sh
export RUST_LOG="warn"
export RUST_BACKTRACE=1
cargo test -- --test-threads=1
#cargo test test_time -- --test-threads=1
