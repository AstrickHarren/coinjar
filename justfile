alias r := run
alias c := check
alias f := fix

fix:
    @ cargo fix --allow-dirty --allow-staged
    @ cargo fmt

check:
    @ cargo check

run Args='check -v':
    @ cargo run -- {{Args}}
