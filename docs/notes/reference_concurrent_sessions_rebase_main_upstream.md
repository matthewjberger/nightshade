---
name: reference_concurrent_sessions_rebase_main_upstream
description: Main can move under you and backward, so fetch right before pushing and compare trees rather than hashes
type: reference
---

Another agent may share this repository, so main can move under you, and it can
move backward as well as forward.

Fetch again right before you push, and read `git diff --stat origin/main..HEAD`.
It must list only the files you touched. When it lists a stranger's file, your
branch sits on a base that is gone: rebase with
`git rebase --onto origin/main <old base> <branch>`, then build again on the new
base.

Compare trees, not hashes. A session that rebases your commits onto another line
gives them new hashes with identical content, and `git diff --stat origin/main main`
is then empty. Pushing there adds a second copy of every commit for no change at
all.
