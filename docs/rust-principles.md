# Rust Principles for Agentic Projects

This document defines the Rust-specific principles that the implementation follows. It supplements the language-agnostic DDD, TDD, and testing principles with Rust idioms, patterns, and constraints.

## Ownership-Driven Design

Rust's ownership system enforces a discipline that maps naturally to DDD: one owner per value, explicit lifetimes, and no shared mutable state by default. Design domain types to align with these rules, not to work around them.

### Ownership Rules

| Rule | Rationale |
|------|-----------|
| Domain types are owned, not borrowed | Aggregates own their state. Pass ownership into constructors; return new instances from mutations. |
| Avoid `Arc<Mutex<T>>` in domain code | Shared mutable state is a design smell. Restructure to pass ownership instead. |
| Use `&T` in function parameters | Functions that only read a value borrow it. Functions that take ownership explicitly say so. |
| `Clone` only at boundaries | Clone data when entering a new ownership context (e.g., spawning a task). Never clone to avoid thinking about lifetimes. |
| No interior mutability in domain types | `Cell`, `RefCell`, and `Mutex` in domain objects indicate a structural problem. Move mutation to the application layer. |

### Move Semantics as Design Signal

When a function takes `T` (not `&T`), it signals: this operation consumes the value. Use this intentionally:

- Builders take `self` and return `Self` — each step is final.
- State machines move from one state type to another — invalid transitions become compile errors.
- Constructors that validate take the raw value and return `Result<ValidType, Error>` — the raw value is consumed.

## Type-Driven Development

Use Rust's type system to make invalid states unrepresentable. The more invariants encoded in types, the less validation code you write at runtime.

### Newtype Pattern

Wrap primitive types in domain-specific newtypes to prevent confusion and enable type-safe validation:

```rust
struct OrderId(Uuid);
struct CustomerId(Uuid);
struct EmailAddress(String);
```

`OrderId` and `CustomerId` are distinct types. A function expecting `OrderId` cannot accidentally receive `CustomerId`. The compiler enforces the domain boundary.

### Parse, Don't Validate

Validate at the boundary. Return a domain type from the constructor or `from_*` function. Once inside the domain, the type is always valid.

```rust
impl EmailAddress {
    pub fn parse(raw: &str) -> Result<Self, ValidationError> {
        // validate once at the boundary
        if raw.contains('@') {
            Ok(Self(raw.to_owned()))
        } else {
            Err(ValidationError::InvalidEmail)
        }
    }
}
```

After `parse` succeeds, `EmailAddress` is always valid. No further checks needed.

### State Machine Types

Encode state transitions in the type system. Invalid transitions fail at compile time, not at runtime:

```rust
struct Order<S> { id: OrderId, state: S }
struct Pending;
struct Confirmed { confirmed_at: DateTime<Utc> }

impl Order<Pending> {
    pub fn confirm(self, at: DateTime<Utc>) -> Order<Confirmed> {
        Order { id: self.id, state: Confirmed { confirmed_at: at } }
    }
}
```

`Order<Pending>` and `Order<Confirmed>` are distinct types. Calling `confirm()` on an already-confirmed order is a compile error.

## Error Handling

Rust errors are explicit in return types. Design error handling as a first-class concern, not an afterthought.

### Two-Layer Error Strategy

| Layer | Crate | Use |
|-------|-------|-----|
| Domain / library errors | `thiserror` | Typed, documented, matchable variants |
| Application / binary errors | `anyhow` | Ergonomic propagation with context |

Domain errors are types. Application errors are context. Never mix them.

### Domain Errors (`thiserror`)

```rust
#[derive(Debug, thiserror::Error)]
pub enum OrderError {
    #[error("order {id} not found")]
    NotFound { id: OrderId },
    #[error("order {id} already confirmed")]
    AlreadyConfirmed { id: OrderId },
    #[error("invalid quantity: must be positive, got {quantity}")]
    InvalidQuantity { quantity: i32 },
}
```

Rules:
- One error enum per domain module, not one global error type.
- Variants carry structured data for programmatic handling.
- Error messages are human-readable without a debugger.
- No `Box<dyn Error>` in domain return types — callers cannot match on it.

### Application Error Propagation (`anyhow`)

```rust
fn process_order(id: OrderId) -> anyhow::Result<Receipt> {
    let order = repository.find(id)
        .context("loading order from repository")?;
    let receipt = order.confirm(Utc::now())
        .context("confirming order")?;
    Ok(receipt)
}
```

Rules:
- Use `?` throughout. No nested `match` for `Ok`/`Err` propagation.
- `.context("…")` adds a layer of explanation at every boundary crossing.
- Do not use `unwrap()` in production paths. Use `expect("invariant: …")` only for true invariants.
- Log errors at the outermost boundary that handles them. Do not log and rethrow.

### Per-Item Error Handling

When processing multiple items, fail per-item, not per-batch:

```rust
let results: Vec<_> = items
    .iter()
    .map(|item| process(item).map_err(|e| {
        tracing::warn!(item_id = %item.id, error = %e, "skipping item");
        e
    }))
    .collect();
```

Collect results; report failures; continue. One bad item does not abort the batch.

## Trait-Based Abstraction (Ports and Adapters)

Traits are Rust's abstraction mechanism. Use them to define ports (what the domain needs) independent of adapters (how those needs are fulfilled).

### Port Definition

Define traits in the domain or application layer. Implementations live in the infrastructure layer:

```rust
// Port: defined in domain, no infrastructure dependency
pub trait OrderRepository: Send + Sync {
    async fn find(&self, id: OrderId) -> Result<Order<Pending>, OrderError>;
    async fn save(&self, order: &Order<Confirmed>) -> Result<(), OrderError>;
}
```

Rules:
- Traits carry only the methods the application actually uses (Interface Segregation).
- `Send + Sync` bounds on traits used with async runtimes.
- Prefer `impl Trait` in function parameters over `Box<dyn Trait>` where possible.
- Use `Box<dyn Trait>` only when runtime polymorphism is required (e.g., storing in a struct field).

### Dependency Injection Without a Framework

Rust has no DI framework. Use constructor injection:

```rust
pub struct OrderService {
    repository: Arc<dyn OrderRepository>,
    mailer: Arc<dyn Mailer>,
}

impl OrderService {
    pub fn new(
        repository: Arc<dyn OrderRepository>,
        mailer: Arc<dyn Mailer>,
    ) -> Self {
        Self { repository, mailer }
    }
}
```

Wire dependencies in `main.rs` or an integration test setup function. Keep `main.rs` thin: parse config, construct adapters, inject into services, run.

### Testing with Trait Fakes

Infrastructure traits enable in-memory fakes for tests:

```rust
#[cfg(test)]
pub struct InMemoryOrderRepository {
    orders: Mutex<HashMap<OrderId, Order<Pending>>>,
}

#[cfg(test)]
impl OrderRepository for InMemoryOrderRepository {
    async fn find(&self, id: OrderId) -> Result<Order<Pending>, OrderError> {
        self.orders.lock().unwrap().get(&id)
            .cloned()
            .ok_or(OrderError::NotFound { id })
    }
    // ...
}
```

Fakes live in test modules or `tests/` — never in production code. Use `mockall` only when a hand-written fake is disproportionately complex.

## Async Patterns (Tokio)

Use async when I/O is involved. Keep domain logic synchronous.

### Rules

| Rule | Rationale |
|------|-----------|
| Domain functions are synchronous | Domain logic has no I/O. Async would add noise without benefit. |
| Repository and adapter methods are async | I/O is async. Traits use `async fn` (or `-> impl Future`). |
| Do not `block_on` inside async code | Blocking an async thread stalls the executor. Use `spawn_blocking` for CPU-bound work. |
| `#[tokio::test]` for async tests | Standard test harness for async unit tests. |
| Use `tokio::join!` for concurrent I/O | Prefer structured concurrency over spawning untracked tasks. |

## Module Structure

```text
src/
├── main.rs              # Entry point: parse config, wire dependencies, run
├── domain/              # Domain types, value objects, aggregates, domain errors
│   ├── mod.rs
│   └── order.rs         # Order aggregate, OrderId newtype, OrderError
├── application/         # Use cases, application services
│   ├── mod.rs
│   └── order_service.rs # OrderService: orchestrates domain + repos
├── infrastructure/      # Concrete adapters (database, HTTP, file)
│   ├── mod.rs
│   └── postgres_order_repository.rs
└── ports/               # Trait definitions (ports the application depends on)
    ├── mod.rs
    └── order_repository.rs
```

### Dependency Direction

```
main.rs → application → domain
                ↑              (ports defined here, implemented in infrastructure)
         infrastructure
```

`domain` has zero external crate dependencies (no `serde`, no `sqlx`, no `tokio` in domain types). The infrastructure layer imports domain and external crates. The application layer imports domain and ports.

### Module Visibility

| Visibility | Use |
|------------|-----|
| `pub` | Public API — types and functions exported to other crates or to `main.rs` |
| `pub(crate)` | Crate-internal visibility — visible across modules but not to external crates |
| `pub(super)` | Visible only to the parent module |
| (none) | Private to the current module |

Prefer the narrowest visibility that works. `pub(crate)` is almost always better than `pub` for internal types.

## Cargo and Build Conventions

### Workspace Layout (multi-crate projects)

```toml
# Cargo.toml (workspace root)
[workspace]
members = ["domain", "application", "infrastructure", "cli"]
resolver = "2"
```

Each crate enforces its dependency policy. `domain` has no external dependencies. `infrastructure` has all of them.

### Feature Flags

Use features only for optional capabilities (e.g., enabling a storage backend). Do not use features to gate required functionality.

### Clippy as a Gate

`cargo clippy -- -D warnings` is required to pass. This means:
- No `clippy::all` suppressions without justification.
- `#[allow(clippy::...)]` is allowed locally with a comment explaining why.
- CI fails on any new warning.

## How This Relates to Project-Level Docs

This document defines Rust-specific patterns. Related documents:

- [`docs/ddd-principles.md`](ddd-principles.md) — language-agnostic DDD principles this implementation follows
- [`docs/tdd-principles.md`](tdd-principles.md) — TDD cycle and quality gate
- [`docs/testing-principles.md`](testing-principles.md) — test structure, naming, assertions
- [`docs/system-design.md`](system-design.md) — module structure, types, implementation order for this project
