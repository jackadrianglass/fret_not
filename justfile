set shell := ["nu", "-c"]

update:
    cargo update

build:
    cargo build --workspace

test:
    cargo test --workspace

run:
    cargo run -p gui
