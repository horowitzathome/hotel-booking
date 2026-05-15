# Testing Principles for Agentic Projects

This document defines how to write, structure, and organize tests in agentic projects. The first part lays out language-agnostic principles. The second part (["Rust Application"](#rust-application)) applies them with Rust specifics.

## Tests Are Specifications

A well-written test answers three questions instantly:

1. **What world does this test live in?** (Setup)
2. **What action triggers the behavior?** (Exercise)
3. **What should the world look like afterward?** (Verification)

If a reader needs more than a few seconds to answer all three, the test is too complex.

## Four-Phase Test Structure

Organize every test into four distinct phases:

1. **Arrange** — build the world the test needs
2. **Act** — trigger the behavior under test
3. **Assert** — check that reality matches expectations
4. **Cleanup** — restore the world (ideally automatic)

Separate phases with blank lines. When the test is clean, phase comments (`// Arrange`) are redundant noise. Remove them.

This applies broadly: never add prose that restates what the code already says. Phase comments, descriptive assertion messages on self-evident assertions, and inline comments narrating obvious logic all violate this rule.

## Test Pyramid

```text
         +----------+
         |   E2E    |  Full pipeline tests
         |  (~5%)   |
        +------------+
        | Integration |  Real I/O, real data
        |  (~15%)     |
       +--------------+
       |  Unit Tests   |  Pure functions, no I/O
       |  (~80%)       |
       +---------------+
```

| Layer | Scope | I/O | Count |
|-------|-------|-----|-------|
| **Unit** | Single function or struct | None — pure logic | ~80% of tests |
| **Integration** | Multi-component with real I/O | Real filesystem, real data | ~15% of tests |
| **E2E** | Full pipeline | Real filesystem, real output | ~5% of tests |

## Mocking Policy

Prefer real implementations over mocks in all layers.

| Principle | Rule |
|-----------|------|
| **Real objects first** | Construct real value objects. They are owned and cheap to create. |
| **Real I/O for integration** | Use real files, real filesystem, real test data. |
| **Mock only at system boundaries** | HTTP clients, database connections, external APIs — these are the only acceptable mock points. |
| **Never mock internal code** | Domain modules, value objects, and services use real implementations. |
| **Hand-write fakes first** | When mocking is necessary, write a simple in-memory implementation. Use mock frameworks only when a fake would be disproportionately complex. |

If a test needs more lines of setup than assertion, that is a signal the production code needs a simpler interface — not that the test needs mocks.

## Test Naming

Tests describe behavior, not implementation. The name should read as a specification.

| Convention | Rule |
|------------|------|
| Test module | `#[cfg(test)] mod tests` inside each source module |
| Test function names | `{action}_should_{outcome}` — describes the expected outcome |
| Parameterized tests | Same function name, data-driven via `#[rstest]` |

## Three-Tier Data Naming Convention

Every value in a test falls into one of three tiers. The naming convention makes each tier explicit.

| Tier | Purpose | Naming Convention | Example |
|------|---------|-------------------|---------|
| **Meaningful** | Directly affects the expected outcome | Role-describing name | `QUANTITY`, `DISCOUNT_RATE`, `HOURLY_WAGE` |
| **Irrelevant** | Required by the API but has no bearing on outcome | `SOME_` / `ANY_` prefix, or anonymous factory | `SOME_EMAIL`, `ANY_ADDRESS`, `create_an_employee()` |
| **Mystery** | Bare literal with no explanation | **Eliminate** | `42`, `"hello@x.com"` |

A test with zero Tier 3 values is self-documenting. The reader scans names alone and knows which data drives the test and which is scaffolding.

### Constants Placement

| Scope | When to Use |
|-------|-------------|
| Module level | Universally irrelevant fixtures (`ANY_ADDRESS`, `SOME_PRODUCT`) |
| Function level | Locally irrelevant values (`SOME_QUANTITY`) or scenario-specific meaningful values (`DISCOUNT_PCT`) |

## Test Data Construction

### Factory Functions

Tests never call production constructors directly with long argument lists. Wrap construction in factory functions owned by the test suite.

```text
BAD:  Order::new(OrderId::new(), CustomerId::new(), vec![item1, item2], 12.50, "FastShip", 2)
GOOD: create_order(OrderId::new(), CustomerId::new(), vec![item1, item2], 12.50, "FastShip", 2)
```

When the constructor signature changes, fix one factory function instead of every test.

### Anonymous Factories

When most fields are irrelevant, create factories that auto-generate irrelevant values and accept only the fields that matter:

```text
create_a_teacher("math")    -- only department matters
create_a_product(price)     -- only price matters
create_an_employee()        -- nothing about the employee matters
```

Behind the scenes, the anonymous factory fills in everything else with unique generated data (counter or UUID). Unique generation prevents data collisions when tests share a datastore or run in parallel.

### Collapse Irrelevant Dependencies

If the test outcome does not depend on an object, the reader should not see it:

```text
BAD:  engine = create_an_engine(); trans = create_a_transmission(); vehicle = create_vehicle(engine, trans, ELECTRIC)
GOOD: vehicle = create_a_vehicle(ELECTRIC)
```

## Derived Expectations

Expected values should be derived from test inputs so the reader can verify correctness by reading the code alone.

```text
BAD:  assert_eq!(payroll.net_amount(), Money::from(142.50))   -- where did 142.50 come from?
GOOD: let gross = HOURS * RATE; assert_eq!(payroll.net_amount(), gross - (gross * TAX_RATE))
```

If an expected value is a function of the inputs, express that function explicitly. The test becomes self-verifying documentation.

## Assertions

### Direct Assertions

Prefer the most specific assertion available. Write assertions that produce clear failure messages.

| Principle | Rule |
|-----------|------|
| Use the most direct assertion available | Pick the assertion that says exactly what you mean |
| One assertion per concern | Multiple assertions on the same result are fine; testing unrelated concerns is not |
| No branching in assertions | No `if/else`, `match`, or loops. Use collection-aware assertions instead |
| Whole-value comparison | Compare complete expected values rather than picking apart fields |

### Stop Re-Testing Other Units

Assert only on the behavior the test owns. Trust that other components' own tests cover them. Build the expected value and compare in one shot rather than asserting on individual fields that belong to another unit.

## Cleanup

### Default to Ephemeral Fixtures

If no persistent side effect occurs (no database writes, no file creation), cleanup should be empty. Use owned values that are dropped when the test ends.

### Never Share Mutable Fixtures

Shared mutable state causes unrepeatable tests, interacting tests, and mystery guests. Each test builds exactly the state it needs via factory functions.

Shared fixtures are acceptable only when immutable — static reference data that no test modifies.

### Automate Persistent Cleanup

For tests that create persistent resources (temporary files, database rows), use RAII wrappers or `Drop` implementations to clean up automatically. Never write per-test teardown logic.

## Testing Vocabulary

All patterns accumulate into a domain-specific testing vocabulary: factory functions, custom assertions, named constants, and `SOME_`/`ANY_` placeholders.

Once the vocabulary exists:
- Writing a new test reuses existing factories and assertions
- Reading is consistent — developers see `create_a_customer(DISCOUNT_PCT)` and understand instantly
- Maintenance is cheap — API changes update one factory, not every test
- Scaling approaches zero cost per new test

Extract shared test utilities into a test helper module or a common `tests/common/mod.rs`. The vocabulary is a project-wide asset.

## Edge Case and Boundary Testing

### Boundary Testing

Every test suite should cover:
- Empty input
- Single item
- Missing state (first run scenario)
- Corrupted or invalid data
- Special characters and Unicode edge cases

### Error Path Testing

- All error scenarios documented in system-design.md have test coverage
- Corrupted data triggers recovery, not crashes
- I/O errors are caught and logged
- Unparseable input produces a warning, not a panic

### State and Idempotency Testing

- First run creates output
- Second run with no changes produces identical output
- New, changed, and removed items are detected
- State round-trips correctly through serialization

## Agent Decision Checklist

When an agent writes or refactors a test, it walks through these checks:

1. **Structure:** Four phases separated by blank lines alone?
2. **No narration:** Free of comments and messages that restate code?
3. **Direct assertions:** Using the most specific assertion available?
4. **Linearity:** No branching or loops in the test body?
5. **Focus:** Only asserting on behavior this test owns?
6. **Whole values:** Comparing complete expected values?
7. **Collection assertions:** Using `assert_eq!` on collections instead of index-based access?
8. **Named patterns:** Recurring verification sequences extracted?
9. **Automatic cleanup:** RAII or Drop handles teardown?
10. **Encapsulated construction:** All objects behind factory functions?
11. **No mystery values:** Every literal is named or declared irrelevant?
12. **Signal vs. noise:** Reader can tell at a glance which values matter?
13. **Transparent expectations:** Expected values derived from inputs?
14. **Zero duplication:** Reusable patterns in the shared vocabulary?

---

# Rust Application

This section applies the principles above with Rust specifics. It shows concrete refactorings, the Rust-specific smell/fix catalog, and assertion patterns. Read the principles section first — the guidance below presumes those rules.

## Assertion Setup

Rust's built-in macros handle most assertions. Use `assert_eq!`, `assert!`, and `assert_matches!` (from `std` or `matches` crate). For parameterized tests, use `rstest`.

```rust
use rstest::rstest;

#[rstest]
#[case(10.0, 5.0, 2.0)]
#[case(20.0, 4.0, 5.0)]
fn division_should_return_quotient(#[case] dividend: f64, #[case] divisor: f64, #[case] expected: f64) {
    assert_eq!(divide(dividend, divisor), expected);
}
```

## Assertion Refactoring Playbook

### Use the Most Direct Assertion Available

**Smell:** Generic `assert!` where a more specific macro would give a better failure message.

**Fix:** Match the assertion to the value being checked.

```rust
// BAD: Generic — failure message just says "assertion failed"
assert!(results.len() > 0);

// BETTER: Specific
assert!(!results.is_empty());

// BEST: Most direct — failure shows the actual value
assert_eq!(results.len(), 1, "expected exactly one result, got {:?}", results);
```

**Rule:** Always choose the most semantically precise assertion. `assert_eq!` and `assert_matches!` produce failure messages that diagnose problems without re-running the test in a debugger.

### Stop Re-Testing Other Units

**Smell:** A test for module A also asserts on internal fields that are the responsibility of module B.

**Fix:** Assert only on the behavior you're actually testing. Build the expected value and compare in one shot.

```rust
// BAD: Testing that Reservation's fields are set inside a Booking test
assert_eq!(reservation.hotel_name(), "Grand Plaza");
assert_eq!(reservation.guest_count(), 2);
assert_eq!(reservation.check_in(), NaiveDate::from_ymd(2025, 6, 1));
assert_eq!(reservation.total_cost(), dec!(800.00));

// GOOD: Build the expected value and compare in one shot
let expected = Reservation::new(
    "Grand Plaza",
    2,
    NaiveDate::from_ymd(2025, 6, 1),
    NaiveDate::from_ymd(2025, 6, 5),
    dec!(800.00),
);
assert_eq!(reservation, expected);
```

**Rule:** Construct the expected result as a complete value and compare with a single `assert_eq!`.

### Flatten All Branching Out of Tests

**Smell:** `if/else` or `match` blocks inside a test body.

**Fix:** Use `assert_eq!` or `unwrap_or_else` to assert and halt. For collections, compare the whole collection.

```rust
// BAD: Conditional logic — did the else branch ever fire?
let items = cart.items();
if items.len() == 2 {
    assert_eq!(items[0].sku(), "ABC-001");
    assert_eq!(items[1].sku(), "XYZ-999");
} else {
    panic!("Cart should contain exactly 2 items");
}

// GOOD: Linear — compare the whole collection
assert_eq!(
    cart.items().iter().map(|i| i.sku()).collect::<Vec<_>>(),
    vec!["ABC-001", "XYZ-999"],
);
```

**Rule:** Tests must be straight-line code. No `if`, no `else`, no loops in assertions. Compare whole collections.

### Name Your Verification Patterns

**Smell:** The same multi-step verification sequence appears in several tests.

**Fix:** Extract into a helper function or custom assertion macro.

```rust
// Repeated in 12 tests:
assert_eq!(cart.items().len(), 1);
assert_eq!(cart.items()[0], expected_item);

// Extracted:
fn assert_cart_contains_only(cart: &Cart, expected: &CartItem) {
    assert_eq!(cart.items(), &[expected.clone()]);
}

// In the test:
assert_cart_contains_only(&cart, &expected_item);
```

**Rule:** Exhaust built-in assertions first. For recurring domain patterns, extract helper functions.

## Cleanup Patterns

### Use RAII for Persistent Cleanup

**Smell:** Explicit cleanup at the end of a test that may not run if the test panics.

**Fix:** Wrap persistent resources in a struct that implements `Drop`.

```rust
struct TempDatabase {
    path: PathBuf,
}

impl TempDatabase {
    fn create() -> Self {
        let path = PathBuf::from(format!("/tmp/test-db-{}", uuid::Uuid::new_v4()));
        // create DB
        Self { path }
    }
}

impl Drop for TempDatabase {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

// Usage in test:
let db = TempDatabase::create();
// test logic using db
// db.drop() called automatically when test ends or panics
```

**Rule:** Never write per-test teardown logic. Build cleanup into a `Drop` implementation.

## Setup Patterns

### Wrap Construction in Factory Functions

**Smell:** Raw struct construction with long parameter lists scattered across dozens of tests.

**Fix:** Wrap each constructor in a factory function owned by the test module.

```rust
// BAD: If Shipment gains a new field, many tests break
let s = Shipment { warehouse: "WH-01", origin: "Seattle", destination: "Portland",
    weight: 12.5, carrier: "FastFreight", days: 2 };

// GOOD: Encapsulated — one place to update
fn create_shipment(warehouse: &str, origin: &str, destination: &str,
    weight: f64, carrier: &str, days: u32) -> Shipment {
    Shipment { warehouse: warehouse.to_owned(), origin: origin.to_owned(),
        destination: destination.to_owned(), weight, carrier: carrier.to_owned(), days }
}
```

### Hide Values That Don't Matter

**Smell:** Tests spell out every field, even when most are irrelevant.

**Fix:** Create anonymous factory functions that auto-generate irrelevant values.

```rust
// BAD: Which of these values affect the test outcome?
let teacher = create_teacher(442, "Maria", "Chen", "Math", "mchen@school.edu", true);
let course = create_course(901, "Algebra II", 4, "B-204", "Fall");

// GOOD: Only the relevant values are visible
let teacher = create_a_teacher("Math");
let course = create_a_course(4);  // only credit hours matter
```

### Signal Irrelevance with SOME_ / ANY_ Constants

```rust
const SOME_NAME: &str = "AnyEmployee";
const SOME_EMAIL: &str = "any@test.com";

fn any_department() -> Department {
    create_a_department()
}

// In the test:
let hours_worked = dec!("40");
let hourly_rate = dec!("75.00");

let emp = create_an_employee();  // everything is irrelevant
let run = PayrollRun::new(any_department(), emp, hours_worked, hourly_rate);
```

## Derive Expected Values from Inputs

```rust
// BAD: Where did 142.50 come from?
assert_eq!(payroll.net_amount(), dec!("142.50"));

// GOOD: Derivation is transparent
let hours = dec!("20");
let rate = dec!("15.00");
let tax_rate = dec!("0.05");

let gross = hours * rate;
assert_eq!(payroll.net_amount(), gross - (gross * tax_rate));
```

## Smell / Fix Quick Reference

| Smell | Fix | Technique |
|-------|-----|-----------|
| `assert!(result.len() > 0)` | `assert!(!result.is_empty())` or `assert_eq!(result.len(), N)` | Direct Assertion |
| Field-by-field assertions | Compare whole values with `assert_eq!` | Whole Value Comparison |
| `if/else` or loops in tests | Flat assertions, whole-collection comparison | Guard Assertion |
| Repeated multi-step verification | Helper function | Custom Assertion |
| Shared mutable fixture | Fresh fixture per test via cheap factories | Fresh Fixture |
| Manual cleanup in test body | RAII wrapper with `Drop` | Registration Pattern |
| Raw struct construction in tests | Wrap in test-owned factory functions | Creation Factory |
| Irrelevant hard-coded values | `SOME_` / `ANY_` constants or anonymous factories | Declarative Irrelevance |
| Hard-coded IDs causing collisions | Generate unique values from counter/UUID | Unique Test Data |
| Visible irrelevant dependencies | Collapse into parent factory | Collapsed Factory |
| Bare numeric/string literals | Named constants with role-based names | Symbolic Constants |
| Opaque expected values | Compute from test inputs | Derived Expectation |
| Same setup in every test | Compose into higher-level factories | Testing Vocabulary |
| Test utilities trapped in one module | Extract to `tests/common/mod.rs` | Shared Test Module |
