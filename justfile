alias t := test
alias r := run
alias c := check
alias f := fix

fix:
    @ cargo fix --allow-dirty --allow-staged
    @ cargo fmt

check:
    @ cargo check

run *Args='check -v':
    @ cargo run -- -f test/example.coin {{Args}}

run-private *Args='check -v':
    @ cargo run -- -f private/journal.coin {{Args}}

test *Args:
    #!/usr/bin/env bash
    Args={{Args}}
    if [ -z "${Args}" ]; then
        cargo test
    else
        cargo test -- --nocapture ${Args}
    fi
