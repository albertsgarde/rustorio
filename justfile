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
    jj bookmark track {{BRANCH}} --remote origin
    jj git push -b {{BRANCH}} --remote origin

push BRANCH:
    jj git fetch
    jj bookmark move {{BRANCH}} --to=@-
    jj git push

pull:
    jj git fetch
    jj new main

publish:
    cargo publish

serve:
    dx serve --package rustorio-website

build-docker:
    docker build . -f docker/website.dockerfile -t rustorio-website

run-docker:
    docker rm rustorio-website --force 2>/dev/null
    docker run --rm --name rustorio-website -p 8080:8080 --mount type=bind,src=./data,dst=/app/data rustorio-website

install-local *ARGS:
    cargo install --path rustorio {{ARGS}} --force

# Set up the AI player project (run once, or to reset)
[working-directory: "ai-player"]
ai-player-setup:
    touch rustorio && rm -r rustorio
    rustorio setup
    mkdir -p logs
    cd rustorio && cargo remove rustorio
    cd rustorio && cargo add --path ../../rustorio
    cd rustorio && cargo build
    cd rustorio && cargo doc -p rustorio --no-deps

# Run the AI player on the tutorial game mode.
#
# Sandboxing: the AI is restricted to a narrow set of allowed tools (see
# ai-player/.claude/settings.json). Specifically it may only:
#   - Read ./src/ and ./target/doc/rustorio/
#   - Edit ./src/bin/tutorial/ (its solution file)
#   - Run `rustorio play tutorial` (to test its solution)
# It cannot edit Cargo.toml, touch the build output, use any MCP tools beyond
# rust-analyzer-lsp, or access the user's global Claude config/settings.
#
# Note: --permission-mode dontAsk means denied actions are silently blocked
# rather than prompting. The allow/deny lists in settings.json are the actual
# enforcement — dontAsk just prevents interactive overrides during the run.
# There is no OS-level sandbox (no seccomp/container); the above is purely
# Claude Code's own permission layer.
[working-directory: "ai-player/rustorio"]
ai-test:
    claude \
        --setting-sources project \
        --strict-mcp-config \
        --settings ../.claude/settings.json \
        --append-system-prompt-file ../CLAUDE.md \
        --permission-mode dontAsk \
        "Begin!" \
        2>&1 | tee "../logs/playtest-`date +%Y%m%d-%H%M%S`.log"

[working-directory: "ai-player"]
ai-test2 GAMEMODE:
    #!/usr/bin/env bash
    set -euo pipefail
    AI_ROOT="$(pwd)"
    DIR_NAME="{{GAMEMODE}}-$(date +%Y%m%d-%H%M%S)"
    mkdir $DIR_NAME
    rustorio setup $DIR_NAME --omit-tutorial --crate-name $DIR_NAME
    cd $DIR_NAME/rustorio
    rustorio new-game --game-mode {{GAMEMODE}} {{GAMEMODE}}
    echo "default_save_game = \"{{GAMEMODE}}\"" >> rustorio.toml
    cargo remove rustorio 
    cargo add --path $AI_ROOT/../rustorio          # Use local version of rustorio
    cargo build
    cargo doc -p rustorio --no-deps             # Generate docs for AI reference
    claude \
        --setting-sources project \
        --strict-mcp-config \
        --settings $AI_ROOT/.claude/settings.json \
        --append-system-prompt-file $AI_ROOT/CLAUDE.md \
        --permission-mode dontAsk \
        "Begin!" \
        2>&1 | tee ../ai.log

ai-test3 GAMEMODE:
    cargo run --bin rustorio dev ai-test -- {{GAMEMODE}}