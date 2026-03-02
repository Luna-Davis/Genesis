# 009 — Git automation (stage + deterministic commit messages)

## Context
Genesis should handle routine git work offline and deterministically. We need staging, status summarization, and rule-based commit messages without user-provided prompts.

## Decision
- Implement `git commit` command:
  - Must run inside a Genesis-managed project (`.genesis` present).
  - Uses `git2` to open/discover repo, summarize changes, stage all, and commit.
  - Commit message rules:
    - Only deletions → `chore: remove <file>` (single) or `chore: remove files`.
    - Only additions → `feat: add <file>` (single) or `feat: add files`.
    - Only modifications → if all in tests => `test: update tests`; otherwise `chore: update <file>` or `chore: update files`.
    - Mixed changes → `chore: update project`.
- Status summarization captures added/modified/deleted/renamed/untracked (used for messaging and future watcher UI).
- Provide optional `--message` to override the generated message.

## Consequences
- Deterministic, offline commit flow aligned with the vision.
- Single command to stage and commit everything; safe no-op when there are no changes.
- Future watcher integration can reuse `status_summary` and `generate_commit_message`.
