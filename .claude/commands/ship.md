Format, lint, commit, and push the current changes.

1. Run the gates the **Gates** paragraph of `CLAUDE.md` names for a change like yours, and fix every warning. Never suppress one and never add an `#[allow]`. Always `cargo fmt --all -- --check` and `cargo clippy --workspace --all-targets -- -D warnings`, then the `just lint-*` check the change reaches, then `cargo test --lib -p <crate>` for the crates you touched. Leave the whole `just lint` and `just test` to CI.
2. Launch one app your change reaches, once, with `just run <app> --capture <path> --capture-frame <n>`, and look at the picture. Run that app and no other.
3. Run `git status` to see what changed.
4. Run `git diff --staged` and `git diff` to understand the changes.
5. Stage the files that belong to this change with `git add`.
6. Write the message to a file and commit with `git commit -F`, because a here-string leaks characters into the subject. Conventional Commits, plain prose, no em-dashes and no semicolons, and say why rather than what.
7. NEVER add Co-Authored-By, Signed-off-by, or any attribution line.
8. Show the result with `git log --oneline -1`.
9. Fetch, confirm the tree holds only the files you touched, then push.
