---
name: code-quality-review
description: >-
  Rust code quality checklist for idiomatic 2026 Rust applications.
  Load when conducting code quality reviews.
compatibility:
  - claude-code
  - opencode
  - github-copilot
metadata:
  version: "1.0"
  author: team
---

## Code Quality Checklist

### Naming (Rust Conventions)

- [ ] Types (structs, enums, traits): `CamelCase`
- [ ] Functions, methods, variables, modules: `snake_case`
- [ ] Constants and statics: `SCREAMING_SNAKE_CASE`
- [ ] Lifetimes: short lowercase (`'a`, `'buf`) — descriptive only when ambiguous
- [ ] No `get_` prefix on accessor methods (Rust convention: `name()` not `get_name()`)
- [ ] No prohibited suffixes: `Manager`, `Helper`, `Utility`, `Handler`, `Processor`, `Base`
- [ ] No abbreviations unless universally understood (`id`, `url`, `http` are fine)
- [ ] Module names match the filename: `src/domain/order.rs` exports the `order` module
- [ ] No `util`, `helper`, `common` module names

### Structs and Data Model

- [ ] Domain structs are immutable by default (fields not `pub` without reason)
- [ ] Newtype pattern used for validated values (`struct OrderId(Uuid)` not bare `Uuid`)
- [ ] `#[derive(Debug, Clone, PartialEq)]` on all value types
- [ ] `#[derive(Serialize, Deserialize)]` only on types that cross boundaries (serde)
- [ ] No `pub` fields on domain structs — expose via accessor methods or builder
- [ ] `Default` derived or implemented only where a meaningful zero-value exists
- [ ] Collections use `Vec<T>` not `&[T]` in owned structs (unless lifetime is obvious)
- [ ] `Option<T>` for optional fields, never `String` holding `"null"`

### Error Handling

- [ ] Domain errors use `thiserror` with descriptive `#[error("...")]` messages
- [ ] Application/main code uses `anyhow::Result` for propagation
- [ ] No `unwrap()` in production code paths (use `?` or `expect()` with explanation)
- [ ] `expect()` with a message is allowed for invariants that should never fail
- [ ] No `panic!()` in library code — return `Result`
- [ ] Error variants are exhaustively handled in `match` (no `_ =>` unless truly exhaustive)
- [ ] Per-item errors: log at WARN, continue to next item
- [ ] Fatal errors: log at ERROR, return `Err` or exit with non-zero code
- [ ] No swallowed errors (every `?` or `match Err` logs or propagates)

### Ownership and Borrowing

- [ ] Prefer borrowing (`&T`, `&str`) over cloning in function signatures
- [ ] Clone only where ownership transfer is genuinely needed
- [ ] No unnecessary `Arc<Mutex<T>>` — prefer ownership passing or `Rc<RefCell<T>>` for single-thread
- [ ] Lifetime annotations only where inference fails — prefer restructuring to eliminate them
- [ ] No `unsafe` blocks without `// SAFETY:` comment explaining the invariant
- [ ] `String` vs `&str`: use `&str` in function parameters, `String` for owned storage

### Traits and Abstraction

- [ ] Infrastructure dependencies are behind traits (ports in hexagonal architecture)
- [ ] Traits are single-purpose (Interface Segregation Principle)
- [ ] Prefer `impl Trait` in function parameters over `dyn Trait` where possible
- [ ] `Box<dyn Trait>` only for runtime polymorphism — justify in a comment
- [ ] No blanket `impl` on types you don't own without strong justification

### Rust Idioms (2024 Edition)

- [ ] Iterator chains preferred over manual `for` loops with mutation
- [ ] `?` operator used throughout (no nested `match` for `Ok`/`Err` propagation)
- [ ] `if let` / `while let` for single-variant matching
- [ ] `match` for multi-variant exhaustive handling
- [ ] `map`, `and_then`, `unwrap_or_else` on `Option`/`Result` chains
- [ ] Pattern destructuring in `let` and function parameters
- [ ] `From`/`Into` implementations for type conversions at boundaries
- [ ] No unnecessary `.to_string()` — use `&str` instead

### Logging

- [ ] `tracing` crate with structured fields: `tracing::info!(field = value, "message")`
- [ ] No `println!` or `eprintln!` in production code
- [ ] Log levels: INFO for progress, WARN for recoverable errors, ERROR for failures, DEBUG for detail
- [ ] Sensitive data never logged (even at DEBUG)
- [ ] Span context set at service entry points

### Functions and Methods

- [ ] Single responsibility
- [ ] Early returns for error/edge cases (`?` operator)
- [ ] Methods under ~30 lines (extract helpers if longer)
- [ ] No side effects in methods named as queries
- [ ] `pub(crate)` for crate-internal visibility, not `pub`

### Module Structure

- [ ] Follows Clean Architecture: `domain/`, `application/`, `infrastructure/`, `ports/`
- [ ] No circular dependencies between modules
- [ ] Domain module has zero external dependencies (no `serde`, no `sqlx`, etc. on domain types)
- [ ] `pub use` re-exports at module root for public API surface

### UTF-8 and Edge Cases

- [ ] All string operations are UTF-8 aware (`chars()`, not `bytes()` for character operations)
- [ ] File I/O reads/writes as bytes or UTF-8 strings explicitly
- [ ] No assumption that input is ASCII

### Testing (see testing-principles.md)

- [ ] No mocks in domain or application layer tests — real types only
- [ ] `mockall` allowed at infrastructure trait boundaries
- [ ] `assert_eq!` / `assert!` / `assert_matches!` with descriptive messages
- [ ] `rstest` for parameterized tests
- [ ] Four-phase structure (Arrange/Act/Assert) separated by blank lines, no phase comments
- [ ] Test modules named `tests` using `use super::*;`
