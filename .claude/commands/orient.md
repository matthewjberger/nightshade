Re-orient me. I've been away from this project and forgot where we are. Reconstruct the current state from the repository and memory, do not guess, and give me a tight briefing so I can pick up where I left off.

Investigate, in this order:

1. What this project is: read the README and the package manifest (Cargo.toml / package.json / etc.) for the one-line purpose and the crate/app name. One or two sentences, no marketing.
2. Where the branch is: current branch, how it compares to the main branch (ahead/behind), and whether the tree is clean. Run `git status`, `git branch --show-current`, `git log --oneline -10`.
3. What changed most recently: summarize the last few commits as a trajectory ("we've been doing X"), not a raw list. If there are uncommitted changes, run `git diff --stat` and `git status` and say what's in flight and whether it looks finished or mid-edit.
4. What's unfinished: search the diff and recently-touched files for `TODO`, `FIXME`, `unimplemented!`, `todo!`, `.skip`, `xfail`, or dated notes. Read `docs/notes/README.md` and the notes it points at for the traps, the app facts and the standing rules of this repo, and your own auto-memory if you carry one.
5. Loose ends: any failing or ignored tests, any obvious next step implied by the last commit or the uncommitted work.

Then report, concisely:
- **Project**: what it is, in a sentence.
- **Where we are**: branch state, last milestone, what the recent commits accomplished.
- **In flight**: uncommitted or unpushed work, and whether it's done or partial.
- **Likely next step**: the single most probable thing we were about to do, phrased as a question I can confirm or redirect.

Do not start changing code. End by asking me what I want to focus on. If arguments are provided, bias the briefing toward that area.
