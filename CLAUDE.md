# CLAUDE.md

This file provides guidance to Claude Code when working with code in this repository.

## General Rules

1. Don't assume. Don't hide confusion. Surface tradeoffs.
2. Minimum code that solves the problem. Nothing speculative.
3. Touch only what you must. Clean up only your own mess.
4. Define success criteria. Loop until verified.

## Project Overview

hotel-booking: Rust REST API for hotel room rental and booking management

**Documentation:**
- Requirements and goals: [`docs/prd.md`](docs/prd.md)
- Architecture, patterns, guardrails: [`docs/system-design.md`](docs/system-design.md)
- Architectural decisions: [`docs/adr/`](docs/adr/)
- Documentation structure: [`docs/documentation.md`](docs/documentation.md)
- Developer quick-reference (ports, URLs, curl examples): [`docs/dev_infos.md`](docs/dev_infos.md)
- Historical implementation log (steps 1–21, design notes, open items): [`docs/implementation_plan.md`](docs/implementation_plan.md)

## Agent Usage (Mandatory)

**Rule:** Always use specialized agents for feature development. Do not implement features directly.

### Pipeline Coordinator

For new features or when unsure which agent to invoke, use the `pipeline-coordinator` agent. It reads `.scratch/` state and routes to the correct specialist.

For direct invocation when the target agent is known, use the agent selection table in the `pipeline-handoff` skill.

**Skip agents for:** git operations, answering questions about the codebase, running one-off commands.

**Use review agents for:** formal code reviews (code quality, tests, security, documentation). "Review changes" or "review code" triggers the review agents, not direct implementation. Reading code to answer a question does not require agents.

### Skills (Portable Workflow Knowledge)

Pipeline logic lives in skills (`.claude/skills/`), not in agent definitions. All three tools (Claude Code, OpenCode, GitHub Copilot) read skills from this location.

| Skill | Purpose |
|-------|---------|
| `pipeline-handoff` | Routing table, handoff conditions, blocking rules, state files |
| `prd-authoring` | PRD format, boundary rules, requirement template |
| `tdd-workflow` | TDD cycle process, design-check decision tree, document ownership |
| `code-quality-gate` | Build/test/lint requirements, completion criteria |
| `review-checklist` | Reviewer output format, feedback tags, review process |
| `code-quality-review` | Rust code quality checklist |
| `test-review` | Test quality checklist, Rust testing conventions |
| `security-review` | Security checklists, threat model, severity, dependencies |
| `design-validation` | Architectural validation checklist for feature approval |
| `new-feature` | Clear scratch directory, start fresh feature context |
| `adr-template` | ADR format, naming conventions, when to create |
| `audit-agents` | Audit agent config for consistency and cross-tool parity |
| `feature-eval` | Score completed features: tests, reviews, retry count |
| `doc-review` | Documentation review checklist, validation categories, review process |
| `doc-sync` | Synchronize documentation with codebase after implementation |
| `lint-docs` | On-demand documentation validation |

### Reference

See [`.claude/agents/README.md`](.claude/agents/README.md) for agent roles, model assignments, and scratch directory lifecycle.

## Toolchain

| Tool | Version | Notes |
|------|---------|-------|
| Rust | stable (1.87+) | Via `rustup`; pin via `rust-toolchain.toml` |
| Cargo | (bundled with Rust) | Build, test, dependency management |
| rustfmt | (bundled) | Code formatting; config in `rustfmt.toml` |
| clippy | (bundled) | Linting; `cargo clippy -- -D warnings` |
| cargo-audit | latest | Dependency vulnerability scanning |

## Build Commands

Always look first at file Justfile which contains many useful commands. If you do not find a command you would need, add it first to Justfile and then use it. If you are unsure, ask. 

Below you find common commands, which also should be available via Justfile. 

```bash
just build                # Build debug binary
just build-release        # Build release binary
just test                 # Run all tests (unit + integration)
just test-verbose         # Run tests with stdout
just fmt                  # Format all Rust files
just fmt-check            # Check formatting (fails if unformatted)
just lint                 # Lint (treat warnings as errors)
just audit                # Audit dependencies for CVEs
just run-dev              # Run the application
just doc                  # Build and open documentation
```

## Database Schema Changes

This project is in development mode. Do not create new migration files. Edit the existing files directly:

- `migrations/0001_initial_schema.up.sql`
- `migrations/0001_initial_schema.down.sql`

Workflow when changing the schema:

```bash
just db-migrate-revert        # roll back the current schema
# edit migrations/0001_initial_schema.up.sql and .down.sql
just db-migrate               # apply the updated schema
just sqlx-prepare             # regenerate .sqlx/ offline query metadata
```

The `.sqlx/` files must be committed after any `query!` / `query_as!` macro change, or the Docker build will fail.

See [`docs/dev_infos.md`](docs/dev_infos.md) for sqlx-cli installation and a full migration command reference.

## Architecture

See [`docs/system-design.md`](docs/system-design.md) for module structure, patterns, guardrails, and implementation details.

See [`docs/rust-principles.md`](docs/rust-principles.md) for ownership-driven design, type-driven development, error handling, and trait-based abstraction.

## Writing Standards

All documentation, comments, and PRDs must follow the writing standards in [`docs/documentation.md`](docs/documentation.md#writing-standards).

## Testing Strategy

- **TDD**: Write failing tests before production code. Bug fixes start with a reproducing test.
- **No mocks in domain**: Domain and application layer tests use real types. No `mockall` or mock libraries for domain logic.
- **`mockall` at boundaries**: Use `mockall` only for infrastructure traits (repository, HTTP client, external service adapters).
- **Test placement**: Unit tests as `#[cfg(test)] mod tests { ... }` inside each module. Integration tests in `tests/` directory.
- **Testing principles**: See [`docs/testing-principles.md`](docs/testing-principles.md) for test structure, naming conventions, and the agent decision checklist.
- **Full details**: See [`docs/system-design.md`](docs/system-design.md) for test pyramid, naming conventions, assertion patterns, and test data.

## Scratch Directory

Agents collaborate through `.scratch/` (git-ignored). One feature at a time. Never use system `/tmp` — use `.scratch/tmp/`.

See [`.claude/agents/README.md`](.claude/agents/README.md) for structure, file lifecycle, templates, and rules.

## Quality Gate

Before code review, run:

```bash
just ci
```

All checks (build, test, format, lint) must pass before invoking reviewers.

## Documentation Updates

When changing the codebase, follow the maintenance rules and prohibited patterns in [`docs/documentation.md`](docs/documentation.md#maintenance-rules).

## Commit Convention

Never commit by yourself, until explicitly advised in a prompt. 

Format: `<type>(<scope>): <subject>`

### Types

| Type | Use When |
|------|----------|
| `feat` | New feature or capability |
| `fix` | Bug fix |
| `docs` | Documentation only (PRD, system-design, ADRs) |
| `style` | Formatting, whitespace, no code change |
| `refactor` | Code change that neither fixes bug nor adds feature |
| `perf` | Performance improvement |
| `test` | Adding or updating tests |
| `build` | Build system, dependencies (`Cargo.toml`, `Cargo.lock`) |
| `ci` | CI/CD configuration |
| `chore` | Maintenance tasks, tooling |

### Scopes

Use the crate or module name (e.g., `domain`, `application`, `infrastructure`, `cli`). Omit scope for cross-cutting changes.

### Subject Line Rules

- Imperative mood: "add feature" not "added feature" or "adds feature"
- Lowercase first letter
- No period at end
- Maximum 50 characters
- Complete the sentence: "This commit will ___"

### Examples

```text
feat(domain): add order validation rule
fix(infrastructure): recover from database connection timeout
docs: add ADR for error handling strategy
test(application): add unit tests for checkout use case
refactor(cli): extract argument parsing into separate module
chore: update .gitignore for IDE files
build: bump serde to 1.0.210
```

### Breaking Changes

Add `!` after type for breaking changes:

```text
feat(domain)!: change OrderId to use UUID instead of integer
```

Include `BREAKING CHANGE:` footer in body explaining migration.
