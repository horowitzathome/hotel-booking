# Country codes use ISO 3166-1 alpha-2

**Status:** Accepted

## Context

The Country model needs a compact, standardised code alongside the country name. Three ISO 3166-1 variants exist. We need to pick one and encode it as a database column type and validation rule.

## Options Considered

1. **ISO 3166-1 alpha-2** (two letters, e.g. `DE`, `US`) — most widely recognised; used by browsers, payment processors, and shipping APIs
2. **ISO 3166-1 alpha-3** (three letters, e.g. `DEU`, `USA`) — more expressive but less familiar in web API contexts
3. **ISO 3166-1 numeric** (three digits, e.g. `276`, `840`) — unambiguous but not human-readable without a lookup table

## Decision

We use ISO 3166-1 alpha-2. Alpha-2 is the most widely recognised form, covers all practical integration needs for this project, and maps cleanly to a `CHAR(2)` database column.

## Consequences

- The `iso_code` field on `Country` is validated to exactly 2 characters via `#[validate(length(equal = 2))]`.
- The database column is `CHAR(2)` — no longer or shorter values are accepted at the DB level either.
- API consumers must supply valid alpha-2 codes; invalid values are rejected with HTTP 400.
- If alpha-3 or numeric codes are ever needed (e.g. integration with a payment processor that requires alpha-3), a migration to `CHAR(3)` and updated validation is required.

## Implementation

Implemented in Step 8 (Countries domain slice). No formal requirement ID.

## References

- `src/models/country.rs` — `iso_code` field with `#[validate(length(equal = 2))]`
- `migrations/0001_initial_schema.up.sql` — `iso_code CHAR(2)` column definition
