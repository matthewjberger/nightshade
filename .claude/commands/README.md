# Commands

Slash commands for this engine, one file per command. Claude Code reads this
directory when a session starts in the repo, so they are checked in and travel
with the code they describe.

| Command | What it does |
|---------|--------------|
| `/intro-nightshade` | Orients a cold session: the architecture, the reference apps, the gates, and the gotchas that cost an hour each. |
| `/orient` | Reconstructs where the work stands from the branch, the recent commits, and what is in flight, then asks what to pick up. |
| `/new-game` | Starts a new game. Writes the design prompt first, opens a worktree, copies the template, finds the art that already exists, and ships a vertical slice before any breadth. |
| `/grow-game` | Grows a game past its slice. Screens on app states, an overworld of levels held as data, progression and story, a second mode, and captures that prove each screen. |
| `/sweep-antipatterns` | Sweeps for the patterns in `docs/antipatterns.md` and refactors each one away, one at a time, building between each. |
| `/sweep` | Sweeps the prose and the comments for the AI-isms that `docs/writing-directive.md` names, and fixes them without touching a code token. |
| `/sanity` | Reasons through recent code by reading it. Runs nothing, and writes no test program. |
| `/evaluate` | Reads every file this session changed and reports what is solid and what needs another pass. |
| `/iterate` | Another pass of the same, this time fixing what it finds rather than reporting it. |
| `/review` | Reviews the branch against main. |
| `/review-pr` | Reviews a GitHub pull request and posts the comments. |
| `/ship` | Runs the gates the change reaches, commits with a written message file, and pushes. |
| `/handoff` | Writes the prompt that lets a fresh session finish what this one was doing. |

## Writing one

A command is a markdown file. The frontmatter carries a `description`, which
is what Claude reads to decide whether the command fits the task. Everything
under it is the instruction, addressed to Claude and not to the reader.

```markdown
---
description: One line. What it does, not what it is.
---

# Title

Numbered steps, in the order they run.
```

Claude Code reads the directory at startup, so a new file needs a fresh
session before it appears. A command in `~/.claude/commands/` is global and
reaches every project. A command here belongs to this repo and wins where the
names collide.
