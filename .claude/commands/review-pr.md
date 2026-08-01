Review a PR and post comments. Accepts an optional PR number as argument: $ARGUMENTS

If no PR number is provided, use the current branch's open PR.

## Phase 1: Identify and Sync

1. Derive the repo owner/name from the git remote: `gh repo view --json nameWithOwner --jq .nameWithOwner`. Use this for all `gh api` and `gh pr` commands.
2. If a PR number was given, use it. Otherwise run `gh pr view --json number,url,headRefName,commits` to get the current branch's PR.
3. Get the base branch: `gh pr view <number> --repo <owner/repo> --json baseRefName --jq .baseRefName`. Use this instead of hardcoding "main" for all diffs.
4. Run `git fetch origin` to ensure the remote refs are up to date.
5. Get the latest commit SHA from the PR: `gh pr view <number> --repo <owner/repo> --json commits --jq '.commits[-1].oid'`
6. Compare local HEAD with the remote branch (`git log --oneline HEAD..origin/<branch>` and `git log --oneline origin/<branch>..HEAD`). If they diverge, tell the user to run `git pull --rebase` and wait for them to confirm before continuing. Do not proceed with a desynchronized branch.
7. After sync, re-fetch the commit SHA. This is the SHA you will use for all comment posts.
8. Get the full list of changed file paths from the PR: `gh api repos/<owner>/<repo>/pulls/<number>/files --jq '.[].filename'`. These are the exact paths you must use when posting comments.

## Phase 2: Independent Review

Spawn a background agent to perform the review. Pass any context the user has provided (e.g. the author's experience level, related discussions, what to focus on).

The agent should:

1. Run `git diff <base>...HEAD` to get the full diff
2. Run `git diff <base>...HEAD --name-only` to list changed files
3. Read each changed file in full for context
4. For each file, flag:
   - Bugs or logic errors (copy-paste bugs, off-by-one, unreachable code, values never reset)
   - Dead code (functions defined but never called, no-op attributes)
   - Commented-out code
   - Debug prints (println!, dbg!, etc.) that should use proper logging
   - Unnecessary complexity (traits without polymorphism, wrapper methods over public fields, unnecessary clones)
   - Type safety issues (magic indices, boolean tuples that should be enums, flag-based state machines that should be enums)
   - Naming issues (typos in identifiers)
   - Anything that doesn't match surrounding code style
5. For each issue, include the exact file path and line number(s)
6. If a file looks fine, just say "LGTM" and move on
7. Be direct. No filler. No em-dashes or double hyphens (--).

Wait for the review to complete.

## Phase 3: Evaluate the Review

Go through each point from the agent and verify it:

1. **Check accuracy**: Read the actual lines referenced. Confirm the claim is true against the current code. Drop anything that's wrong.
2. **Check line numbers against the remote diff**: The GitHub API can only place comments on lines that appear in the diff. For large files where patches may be truncated, check `gh api repos/<owner>/<repo>/pulls/<number>/files --jq '.[] | select(.filename | contains("<file>")) | .patch'` to find the actual remote line numbers. Local line numbers may not match if the patch context differs.
3. **Assess severity**: Categorize as Bug, Design, or Cleanup.
4. **Check tone**: Every comment must be diplomatic and constructive. No "you should have" or "this is wrong." Frame suggestions as alternatives. No em-dashes or double hyphens (--).

Present the evaluated findings to the user as a table with file, line(s), category, and description. Ask the user which comments to post and whether any need editing.

## Phase 4: Draft and Iterate

For each comment the user wants to post:

1. Show the full comment text with the target file and line range.
2. Wait for the user to approve or request changes.
3. Revise as needed until the user says to post.
4. For design suggestions, include code snippets showing the concrete alternative so the reviewer has a clear path forward.
5. The user may want to post in batches or one at a time. Follow their lead.

Do not post anything until the user explicitly approves the final text.

## Phase 5: Post Comments

Before posting, verify the PR commit SHA has not changed since Phase 1 (force-pushes can happen mid-review). If it changed, stop and re-sync from Phase 1.

For each approved comment:

1. Use the commit SHA from Phase 1 (re-verified above).
2. Use the full repo-relative file path from the Phase 1 file list.
3. Post as line comments using `gh api repos/<owner>/<repo>/pulls/<number>/comments --method POST`. Use `start_line` and `line` for multi-line comments. Always use `side="RIGHT"` for comments on added/changed lines.
4. If a comment fails to post (422 line resolution error), check the remote patch to find the correct line numbers in the diff hunk. Try the corrected lines. If the lines are outside any diff hunk, fall back to a general PR comment referencing the file and line.
5. After posting, verify each comment landed on the correct lines by checking the API response.

## Rules

- Never post without showing the user first and getting explicit approval
- No em-dashes or double hyphens (--) in any comment text
- Include code snippets for design suggestions so the path forward is concrete
- Flag bugs clearly but diplomatically ("looks like" rather than "this is wrong")
- When multiple issues relate (e.g. removing a trait makes wrapper methods unnecessary), connect them so the reviewer sees the full picture
- Comments about dead code, commented-out code, and debug prints can be brief
- If the PR is force-pushed at any point during the review, all prior work may be invalidated. Re-sync and re-verify before posting.
