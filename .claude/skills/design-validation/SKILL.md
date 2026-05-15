---
name: design-validation
description: >-
  Architectural validation checklist for feature approval.
  Load when validating that features fit into the existing architecture.
compatibility:
  - claude-code
  - opencode
  - github-copilot
metadata:
  version: "1.0"
  author: team
---

## Design Principles

Apply these principles when evaluating features:

1. **Clean Architecture layers** — domain is the center; application orchestrates; infrastructure implements ports.
2. **Consistency over novelty** — match existing patterns unless there is a compelling reason.
3. **Explicit dependencies** — every integration point documented; no hidden coupling via global state.
4. **Granular failure** — errors should be as granular as possible; use typed error enums.
5. **Types for data, traits for behavior** — keep the data model clean and the behavioral abstractions thin.
6. **Test-data driven** — every input handling change must work against test data.

## Validation Checklist

Before approving a feature for implementation:

### Architectural Fit

- [ ] Feature aligns with project goals
- [ ] Feature not in Non-Goals or Out of Scope
- [ ] Module placement follows existing Clean Architecture layers (see `docs/system-design.md`)
- [ ] Domain types use `struct`/`enum` with no framework annotations (no `serde`, no `sqlx`)
- [ ] Infrastructure dependencies hidden behind traits (ports)
- [ ] Error handling uses `thiserror` for domain errors, `anyhow` for application orchestration
- [ ] No circular dependencies between modules
- [ ] Integration points identified and documented

### Rust and Clean Architecture Alignment

See `docs/rust-principles.md` for full principles.

- [ ] Domain structs are immutable (fields not `pub` without justification)
- [ ] Newtype pattern used for validated primitive values (`OrderId(Uuid)` not bare `Uuid`)
- [ ] Traits define behavior contracts at module boundaries
- [ ] No `unsafe` code without explicit `// SAFETY:` justification
- [ ] No `unwrap()` in production code — `?` or `expect("invariant")` instead
- [ ] `From`/`Into` implementations at layer boundaries for type conversions

### Security by Design

- [ ] Credentials handled per existing patterns (environment/config, not hardcoded)
- [ ] Input validation at system boundaries (not assumed inside domain)
- [ ] Error messages don't leak sensitive data
- [ ] Logging follows redaction patterns (no secrets in `tracing` fields)
- [ ] Network operations use TLS (no `danger_accept_invalid_certs`)

### Reliability by Design

**Robustness:**
- [ ] Failure modes enumerated
- [ ] Failure in one item does not prevent processing others
- [ ] Corrupted data/state files handled gracefully (not panic)
- [ ] Unparseable inputs handled — log and return `Err`, not `unwrap()`
- [ ] I/O errors caught and returned via `Result`
- [ ] Resource limits defined (bounded channels, connection pools)
- [ ] Graceful shutdown behavior specified (if applicable)

**Idempotency:**
- [ ] Running twice with no changes produces identical output
- [ ] State and output files consistent after every run
- [ ] No partial writes leave the system in a broken state

### Understandability

**Decomposition:**
- [ ] Feature maps to a clear module and type
- [ ] Single responsibility: one struct/function does one thing
- [ ] No hidden dependencies between modules (no `use crate::infrastructure` from `domain`)
- [ ] Component can be understood in isolation

**Clear Interfaces:**
- [ ] Functions accept typed parameters (no raw `String` where a newtype applies)
- [ ] Return types express failure clearly (`Result<T, E>` for fallible, `Option<T>` for absence)
- [ ] Trait methods have descriptive signatures without implementation leakage

**Predictable Behavior:**
- [ ] Error handling matches the table in system-design.md
- [ ] State changes are explicit (no hidden mutation via `RefCell` or `Mutex`)
- [ ] Side effects limited to defined output locations

### Data Model Integrity

- [ ] New types follow naming conventions
- [ ] State file schema remains backward-compatible (or migration path defined)
- [ ] Serde serialization round-trips correctly
- [ ] UTF-8 encoding maintained throughout string processing

### Testability

- [ ] Domain logic unit-testable without filesystem or network
- [ ] Infrastructure traits mockable with `mockall`
- [ ] Integration tests can use `tempfile::tempdir()` for file I/O
- [ ] Edge cases coverable by parameterized tests (`rstest`)
