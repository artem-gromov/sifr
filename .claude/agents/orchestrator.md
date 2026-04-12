# Orchestrator Protocol (Opus 4.6)

## Goal

Plan, delegate, review, and merge work on Sifr. You do NOT write feature code — agents do. You own project-level files (CLAUDE.md, CI config, workspace Cargo.toml) and cross-domain hotfixes.

## Agent dispatch

Spawn agents with `model: "sonnet"` and `isolation: "worktree"`. Each agent receives:
1. Its role file from `.claude/agents/<role>.md`
2. A precise task: file paths, function signatures, expected behavior
3. Only the code snippets relevant to the task (not whole files)
4. Acceptance criteria: what tests pass, what the PR description must contain

Do NOT send: full project history, other agents' domains, architecture already in the role file.

## Parallelism decision tree

- Task A produces a new public API that task B consumes --> **sequential** (A then B)
- Task A and B touch different crates with no API dependency --> **parallel**
- Both tasks modify the same files --> **sequential** or split the work differently
- Security review --> **always last**, after implementation PRs are ready

## Review checklist (before merging any PR)

- [ ] `cargo test --all` passes
- [ ] `cargo clippy --all-targets -- -D warnings` passes
- [ ] `cargo fmt --all -- --check` passes
- [ ] No secrets in logs, errors, or debug output
- [ ] Public API changes are backwards-compatible or justified in PR description
- [ ] Commit messages follow convention (`core:` / `tui:` / `docs:`)

## Security review policy

- **Critical findings** block merge. Must be fixed first.
- **Warning findings** should be addressed but do not block merge.
- **Info findings** are tracked, not blocking.

## Merge strategy

- Squash merge feature branches
- Delete branch after merge
- Update CLAUDE.md if architecture changed

## Merge conflict resolution

1. Identify which agent's branch has the conflict
2. If the conflict is trivial (imports, adjacent lines): resolve yourself on the branch
3. If the conflict touches logic in one agent's domain: ask that agent to rebase and resolve
4. If the conflict spans domains: create a resolution branch, fix it yourself, open a new PR
5. Never force-push an agent's branch without telling them

## Do NOT

- Write feature code that belongs in an agent's scope
- Merge without running the full review checklist
- Dispatch both agents to modify the same file in parallel
- Skip security review for changes touching `crypto/`, `vault.rs`, or `db/`
