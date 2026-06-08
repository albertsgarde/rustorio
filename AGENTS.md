# Checking

To check your code for errors, run `just check --strict`. It will run clippy,
fmt, doc and nextest.
Sometimes it can make sense to run them separately, but for a total check, this is the best way,
since it will stay up to date with new kinds of checks.

# Dependencies

When adding dependencies, always make that very clear to me.
Also, all configuration for the dependency should be specified in the workspace `Cargo.toml` file, not in the individual save game `Cargo.toml` files.
The individual crate `Cargo.toml` files should then refer to that.

# Version control
I use JJ for version control, so use `jj` commands instead of `git` commands. For example, use `jj st` instead of `git status`.

# Crates
The `rustorio` crate contains all the user facing code and APIs, while `rustorio-engine` contains all the behind the scenes code.
All parts of `rustorio-engine` that a user should interact with are re-exported in `rustorio`.
Keep this in mind when writing documentation, since the audience will differ between the two crates.
If it is in `rustorio-engine` and is not exported in `rustorio`, its documentation should be targeted at developers and modders, not players.

# Documentation
Examples in documentation should be valid doc-tests, but should hide any code that is not relevant to the example itself, e.g. most `use` statements.
I recommend using the `TokenOfCreation` to create the required resources in a hidden part of the example and then use them in the visible part.
