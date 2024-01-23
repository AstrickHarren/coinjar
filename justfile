fix: 
    cargo fix --allow-dirty
    cargo clippy --fix --allow-dirty
    cargo fmt

fmt: 
    cargo fmt

test: 
    cargo nextest run
