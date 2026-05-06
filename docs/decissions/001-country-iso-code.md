# ADR 001 — Country codes use ISO 3166-1 alpha-2

## Status
Accepted

## Decision
Country codes are stored and exposed as two-letter strings per the **ISO 3166-1 alpha-2** standard
(e.g. `DE` for Germany, `ES` for Spain, `US` for United States).

## Rationale
- Alpha-2 is the most widely recognised form — used by browsers, payment processors, and shipping APIs.
- Two characters are compact and human-readable without a lookup table.
- ISO 3166-1 also defines alpha-3 (three letters) and numeric codes, but alpha-2 covers all practical
  integration needs for this project.

## Consequences
- The `iso_code` field on the Country model is validated to be exactly 2 uppercase ASCII letters.
- The database column is `CHAR(2)`.
- API consumers must supply valid alpha-2 codes; invalid values are rejected with HTTP 400.
