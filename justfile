alias c := check
alias f := fix
alias r := run

fix:
    @ cargo fix --allow-dirty --allow-staged
    @ cargo fmt

run:
    @ cargo run -- -j journal.coin

check:
    @ cargo check
