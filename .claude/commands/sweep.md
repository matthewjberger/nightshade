Sweep the whole repository's prose for AI-isms and fix them, following the No-AI-isms writing directive. Prose here means documentation and every hand-written comment, not code.

The directive is the authority. Read it in full first at `docs/writing-directive.md` (14 sections). Apply every section. Per its section 14, scan each whole file, not only the part you happen to be looking at.

## Guardrails, non-negotiable

- Edit only comment text and markdown prose. Never change a code token, identifier, attribute, string literal, or anything inside a fenced code block, an EBNF/grammar block, or a table. In a repo that self-hosts or has golden-output tests, comments are stripped before those outputs, so a comment-only edit is safe and a code edit is not.
- Make the smallest edit that removes the violation and keeps the exact technical meaning. Do not add words, do not explain more, do not flatten the author's voice.
- Preserve the terse, declarative voice already in the files. If a fix would change meaning or voice, flag it rather than rewriting (directive 14.4).
- Keep each file's existing comment wrap width so a formatter does not reflow the file.
- Carve-out: a colon that introduces a real list, a code block, or a definition stays, and semicolons stay inside code, grammar notation, and tables. The target is prose colons and semicolons that join two clauses, plus every AI-ism in the directive.

## Procedure

1. Read the directive in full.
2. Enumerate what to sweep. All markdown (`*.md`, including the README and any `docs/`), and comments in every source language present (`*.rs`, `*.frost`, `*.c`, `*.h`, `*.go`, `*.ts`, `*.py`, and so on), plus prose CLI help and error strings. Exclude `target/`, `node_modules/`, `vendor/`, `dist/`, `build/`, `.git/`, lockfiles, and generated or third-party files.
3. Size it. If the set is a handful of files, do it inline. If it is dozens, fan out: launch parallel general-purpose agents, each assigned a disjoint group (for example one agent for the docs, one for each large source directory). Give each agent the directive path, the guardrails above, and its exact file list. Tell each to report the categories it fixed and to touch comment or prose text only.
4. Apply the directive per file. Section 1 punctuation first (em dashes, en dashes, arrows, ellipsis, prose colons and semicolons, colon-formatted headers, emoji, Title Case headers to sentence case). Then sections 2 through 11. For code comments, section 11 is the one that bites: obvious-narration comments, tutorial narration ("First, we", "Now we can", "As you can see"), "This function is responsible for", "Helper function to", "handles the logic for", "Note that" openers, and docstrings that only restate the signature. Use the section 12 heuristics (anywhere, negation, delete, specificity, voice, density) to decide what carries no information.
5. Verify nothing but prose moved. Run `git diff` and confirm every changed line is a comment or a markdown line. A quick check is to filter the diff for added and removed lines and see that each is a comment for that language. If the repo builds, run its build, linter, and tests. If it self-hosts or has golden-output tests, run the fixpoint or golden tests to prove the emitted output is byte-identical, which is the real evidence that only comments changed.
6. Report the categories of violations found and fixed per area, briefly. Do not list every instance unless asked.
7. Do not commit or push unless asked, or unless this was run as part of `/ship`.

If the directive was expanded between runs, sweep again. A second pass with the fuller ruleset catches what a narrower first pass missed, which is the pattern that produced this directive.
