---
name: test-review
description: >-
  Test quality checklist, mocking policy, and test organization conventions
  for Rust applications. Load when conducting test reviews.
compatibility:
  - claude-code
  - opencode
  - github-copilot
metadata:
  version: "1.0"
  author: team
---

## Testing Pyramid

| Level | Proportion | Scope | Speed |
|-------|------------|-------|-------|
| Unit tests | ~80% | Single module in isolation | Fast (<100ms) |
| Integration tests | ~15% | Multi-module with real I/O | Medium (<5s) |
| End-to-end tests | ~5% | Full binary execution | Slow (<30s) |

## Test Quality Checklist

### Mocking Policy

- [ ] No mock libraries in domain or application layer tests — real types only
- [ ] `mockall` allowed exclusively at infrastructure trait boundaries (repositories, HTTP clients)
- [ ] Real file I/O used in integration tests via `tempfile::tempdir()`
- [ ] If a test requires complex setup, that signals the production code needs a simpler interface

### Test Placement

- [ ] Unit tests live in `#[cfg(test)] mod tests { use super::*; ... }` inside the module file
- [ ] Integration tests live in `tests/` directory at project root
- [ ] Test helper utilities shared across test files go in `tests/common/mod.rs`
- [ ] No test logic in production code (`#[cfg(test)]` in `src/` is fine, in `src/lib.rs` for helpers is acceptable)

### Rust Test Structure

- [ ] `assert_eq!(actual, expected)` — actual first, expected second (Rust convention)
- [ ] `assert!(condition, "message explaining what failed")` for boolean checks
- [ ] `assert_matches!(value, Pattern)` for enum variant matching (stable since 1.82)
- [ ] `#[should_panic(expected = "message")]` for explicit panic tests
- [ ] No `unwrap()` in test assertions — prefer `expect("reason")` or proper `assert_eq!`
- [ ] `pretty_assertions::assert_eq!` for readable struct comparison output

### Test Structure (Four-Phase)

- [ ] Arrange/Act/Assert separated by blank lines (no phase comments)
- [ ] One logical assertion per test (multiple `assert_eq!` calls on same result are fine)
- [ ] Straight-line code: no `if/else`, `match`, or loops in test bodies
- [ ] Test functions named to describe behavior: `test_order_total_includes_tax` (not `test1`)
- [ ] `#[ignore]` used with a comment explaining when/why to re-enable
- [ ] Tests are independent — no shared mutable state, no ordering dependencies

### Test Data Naming

- [ ] Meaningful values named by role (`DISCOUNT_RATE`, `TAX_RATE`) — Tier 1
- [ ] Irrelevant values use `SOME_` / `ANY_` prefix or anonymous helpers (`any_order()`) — Tier 2
- [ ] No mystery literals (bare `42`, `"test@example.com"`) — Tier 3 eliminated
- [ ] Expected values derived from inputs, not hard-coded magic numbers
- [ ] Object construction wrapped in `fn create_*()` helpers, not inline

### Parameterized Tests (rstest)

- [ ] `#[rstest]` used for repetitive test cases (not copy-paste tests)
- [ ] Test case names descriptive when using `#[case]`
- [ ] Each parameter combination is independently meaningful

### Edge Case Coverage

- [ ] All documented edge cases from prd.md have dedicated test cases
- [ ] Empty input tested
- [ ] Single item tested
- [ ] Missing state file (first run)
- [ ] Corrupted/invalid input tested
- [ ] Unicode edge cases tested (emoji, RTL, null bytes where relevant)

### Error Path Testing

- [ ] All `Err` variants from system-design.md have test coverage
- [ ] Corrupted data triggers graceful recovery (not panic)
- [ ] Missing configuration produces descriptive error
- [ ] I/O errors are caught and returned as `Err`

### State and Idempotency Testing

- [ ] First run creates output
- [ ] Second run with no changes produces identical output
- [ ] New items are detected and processed
- [ ] State file round-trips correctly through serialization

## Test File Organization

### Naming Conventions

- Unit test modules: `mod tests { ... }` inside `src/**/*.rs`
- Integration test files: `tests/<feature>_test.rs` or `tests/<feature>/mod.rs`
- Test helper functions: `fn any_<type>() -> <Type>` for irrelevant instances
- Test helper functions: `fn create_<type>_with_<attribute>() -> <Type>` for meaningful instances

### Integration Test Structure

```rust
// tests/order_processing_test.rs
mod common;  // shared helpers

#[test]
fn test_full_order_pipeline_succeeds() {
    // Arrange
    let dir = tempfile::tempdir().unwrap();
    let order = common::any_order();

    // Act
    let result = process_order(&order, dir.path());

    // Assert
    assert!(result.is_ok());
}
```

## Common Issues to Flag

### [AUTOFIX] Issues

- Missing `use super::*;` in unit test module
- Wrong assertion order (`assert_eq!(expected, actual)` instead of `assert_eq!(actual, expected)`)
- `unwrap()` in test assertions without message
- Non-descriptive test name (`test1`, `test_works`)
- Missing edge case in parameterized test table

### [ESCALATE] Issues

- No integration test for external service boundary
- Test coverage below 80% on domain layer
- No concurrent access testing for shared state

### [CLARIFY:security-reviewer] Issues

- Test exposes sensitive data handling patterns
- Error message content needs security review
