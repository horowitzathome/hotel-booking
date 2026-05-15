---
name: code-quality-gate
description: >-
  Build, test, format, and lint requirements that must pass before
  code review. Load when checking implementation completeness or
  running the quality gate.
compatibility:
  - claude-code
  - opencode
  - github-copilot
metadata:
  version: "1.0"
  author: team
---

## Quality Gate

Before invoking reviewers, all checks must pass.

### Required Checks

| Check | Command | What It Verifies |
|---|---|---|
| Build | `cargo build` | Project compiles |
| Test | `cargo test` | All tests pass |
| Format | `cargo fmt --check` | Code follows rustfmt style |
| Lint | `cargo clippy -- -D warnings` | No clippy warnings |

### Fix Formatting

```bash
cargo fmt
```

Formats all Rust files. Run before `cargo fmt --check`.

### Fix Clippy Issues

```bash
cargo clippy --fix --allow-dirty
```

Auto-fixes many clippy warnings. Review changes before committing.

## Configuration Sync

After implementing a feature that adds or changes configuration:

- [ ] New config fields appear in the config struct
- [ ] Config struct implements `serde::Deserialize`
- [ ] prd.md configuration table updated (if user-facing property)
- [ ] Default values consistent across all locations

## Completion Criteria

A feature is complete when:

- [ ] All TDD cycles finished
- [ ] All tests pass (`cargo test`)
- [ ] Code formatted (`cargo fmt`)
- [ ] Format check passes (`cargo fmt --check`)
- [ ] Project builds (`cargo build`)
- [ ] No clippy warnings (`cargo clippy -- -D warnings`)
- [ ] Configuration synced (if config changed)
- [ ] All four reviewers approve
- [ ] No pending escalations (or human approved)
