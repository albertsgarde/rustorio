fmt:
    cargo fmt

# If you don't have nextest installed, you can get it via `cargo install cargo-nextest`
test *ARGS:
    cargo nextest r {{ARGS}}

alias t := test

check STRICT="":
    cargo clippy --all --all-targets {{ if STRICT != "" { "-- -D warnings" } else { "" } }}
    {{ if STRICT != "" { "RUSTFLAGS=\"-Dwarnings\" RUSTDOCFLAGS=\"-Dwarnings\"" } else { "" } }} cargo doc --no-deps
    cargo fmt --check --all
    just test

doc *FLAGS:
    cargo doc -p rustorio -p rustorio-engine -p rustorio-derive --no-deps {{ FLAGS }}

create-remote-branch BRANCH:
    jj git fetch
    jj bookmark create {{BRANCH}} -r @-
    jj git push -b {{BRANCH}} --allow-new --remote origin

push BRANCH:
    jj git fetch
    jj bookmark move {{BRANCH}} --to=@-
    jj git push

pull:
    jj git fetch
    jj new main

publish:
    cargo publish --allow-dirty

install-local:
    cargo install --path rustorio
