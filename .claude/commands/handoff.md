Write the prompt that lets a fresh session finish what this one was doing, put it
on my clipboard, and tell me to compact.

The next session will have none of this conversation. It gets exactly what you
write and the repository. Write for that reader.

## Gather first, do not guess

1. `git status --short`, `git log --oneline -8`, `git branch --show-current`,
   and whether the branch is pushed. Name the last pushed commit.
2. `git diff --stat` for uncommitted work, and read the actual diff of anything
   you are about to describe. Do not describe a change from memory of writing
   it.
3. The task list, if this session used one, and which items are open.
4. Any gate this session ran and what it answered: test counts, lint, format,
   and whether a run is still in flight.

## Then write the prompt

Plain prose and fenced commands, no preamble, no "here is your handoff". It must
stand alone. Include:

**The task.** What is being built and why, in a few sentences. The next session
must be able to act without knowing this conversation happened.

**The state of the tree**, split three ways, because conflating these is how a
handoff misleads:
- committed and pushed (name the commit)
- uncommitted and verified (say exactly what verified it)
- uncommitted and NOT verified (say what would verify it)

Never write that something is done when a gate for it is still running or was
never run. If a suite was killed, interrupted, or timed out, say so.

**How to reproduce**, as commands that can be pasted. If a failure is the
subject, give the exact command and the exact text it prints.

**Dead ends.** Every approach tried that did not work, and why. This is the part
that saves the most time and the part most often left out. Say plainly: do not
retry this shape.

**The design**, with `file:line` anchors for every place that has to change, and
the names of the functions involved. Count the call sites where that matters.

**What could go wrong**, if you know: a check that fires where it should not, a
table that has to stay in step, an ordering that matters.

**The project's standing rules**, gathered from `CLAUDE.md`, the notes under
`docs/notes/`, and what this session was corrected on. Do not invent them and do not pad with
generic advice. The ones that keep mattering are usually: what must stay in
parity, what to measure rather than assert, what comments are for, and what a
commit message may not contain.

**The gates**, as exact commands with the numbers they should answer, so the next
session knows what green looks like rather than guessing.

## Deliver it

Write the prompt to a file under the session scratch directory if one exists,
otherwise the system temp directory, then copy it to the clipboard:

```powershell
Get-Content <path> -Raw | Set-Clipboard
```

On macOS use `pbcopy`, on Linux `xclip -selection clipboard` or `wl-copy`.
Confirm the character count so I know it landed.

Then tell me, in one line, to run `/compact` and paste. You cannot trigger the
compaction yourself: `/compact` is a built-in command, so it has to come from
me. Say that plainly rather than implying you did it.

Keep the report around the prompt to two or three lines. The prompt is the
deliverable. Do not summarize it back to me.

If arguments are given, treat them as what I want the next session to work on,
and bias the prompt toward that rather than toward whatever was most recent.
