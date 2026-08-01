Review the current branch's changes against main.

## Phase 1: Independent Review

Spawn a background agent to perform the review. The agent should:

1. Run `git diff main...HEAD` to get the full diff
2. Run `git diff main...HEAD --name-only` to list changed files
3. Read each changed file in full for context
4. For each file, flag:
   - Bugs or logic errors
   - Missing edge cases
   - Unnecessary complexity
   - Naming or clarity issues
   - Anything that doesn't match surrounding code style
5. If a file looks fine, just say "LGTM" and move on
6. Be direct. No filler.

Wait for the review to complete.

## Phase 2: Evaluate the Review

Once the review comes back, go through each point and assess whether it is fair given the full context of this conversation: the design decisions, the constraints, and the tradeoffs that led to the code being written this way.

For each review point, respond with one of:
- **Valid.** The reviewer caught a real issue. Note what should be fixed.
- **Debatable.** A reasonable point, but there is context the reviewer did not have. Explain it.
- **Unfair.** The reviewer is wrong or missing context. Explain why.

Present the final output as the review points with your assessments inline.
