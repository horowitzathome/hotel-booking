# REST API Specification

## Conventions

### Base URL

All domain endpoints are prefixed with `/api/v1/`. Observability endpoints are at the root.

```
/health
/metrics
/api/v1/countries
/api/v1/addresses
/api/v1/managers
/api/v1/persons
/api/v1/houses
/api/v1/houses/{house_id}/calendar
/api/v1/bookings
```

### HTTP Methods and Status Codes

| Action | Method | Success status |
|---|---|---|
| List resources | `GET /resources` | 200 |
| Get one | `GET /resources/{id}` | 200 |
| Create | `POST /resources` | 201 + `Location: /api/v1/resources/{id}` header |
| Update (full replace) | `PUT /resources/{id}` | 200 |
| Update (partial) | `PATCH /resources/{id}` | 200 |
| Delete | `DELETE /resources/{id}` | 204 |
| State transition | `POST /resources/{id}/action` | 200 |

Error status codes:

| Situation | Status |
|---|---|
| Resource not found | 404 |
| Validation failure (missing/invalid fields) | 400 |
| Business rule violation (e.g. book non-Rentable days) | 422 |
| Referential constraint (e.g. delete country with addresses) | 409 |

### Date Format

All dates are calendar days in ISO 8601 format: `YYYY-MM-DD`. No timestamps or time zones are used.

### Date Ranges

- **Writes** (create, update): date range goes in the **request body**.
- **Reads / deletes**: date range goes as **query parameters** `from` and `to`.

### Field Naming

JSON fields use `snake_case` throughout, consistent with Rust/serde defaults.

### Error Response Body

All error responses use:
```json
{
  "error": "Human-readable message",
  "details": ["field: reason", "..."]
}
```
`details` is omitted when there are no per-field validation messages.

---

## Health & Observability

### `GET /health`

Returns 200 when the service is up. Used by Kubernetes liveness and readiness probes.

**Response 200:**
```json
{ "status": "ok" }
```

### `GET /metrics`

Returns Prometheus-format text. Scraped by Grafana.

---

## Countries

### `GET /api/v1/countries`

**Response 200:**
```json
[
  { "id": 1, "name": "Germany", "iso_code": "DE" },
  { "id": 2, "name": "Spain",   "iso_code": "ES" }
]
```

### `GET /api/v1/countries/{id}`

**Response 200:**
```json
{ "id": 1, "name": "Germany", "iso_code": "DE" }
```

### `POST /api/v1/countries`

**Request body:**
```json
{ "name": "Germany", "iso_code": "DE" }
```

**Response 201:**
```json
{ "id": 1, "name": "Germany", "iso_code": "DE" }
```

### `PUT /api/v1/countries/{id}`

**Request body:**
```json
{ "name": "Deutschland", "iso_code": "DE" }
```

**Response 200:** same shape as GET.

### `DELETE /api/v1/countries/{id}`

**Response 204.** Returns 409 if any address references this country.

---

## Addresses

### `GET /api/v1/addresses/{id}`

**Response 200:**
```json
{
  "id": 10,
  "street": "Hauptstraße",
  "number": "12",
  "postcode": "10115",
  "city": "Berlin",
  "province": null,
  "country": { "id": 1, "name": "Germany", "iso_code": "DE" }
}
```

### `POST /api/v1/addresses`

**Request body:**
```json
{
  "street": "Hauptstraße",
  "number": "12",
  "postcode": "10115",
  "city": "Berlin",
  "province": null,
  "country_id": 1
}
```

**Response 201:** same shape as GET.

### `PUT /api/v1/addresses/{id}`

**Request body:** same shape as POST.

**Response 200:** same shape as GET.

### `DELETE /api/v1/addresses/{id}`

**Response 204.** Returns 409 if any house references this address.

---

## Managers

### `GET /api/v1/managers`

**Response 200:**
```json
[
  { "id": 1, "first_name": "Hans", "last_name": "Müller", "email": "hans@example.com", "phone": "+49123456789" }
]
```

### `GET /api/v1/managers/{id}`

**Response 200:** single manager object (same shape as list item).

### `POST /api/v1/managers`

**Request body:**
```json
{ "first_name": "Hans", "last_name": "Müller", "email": "hans@example.com", "phone": "+49123456789" }
```

**Response 201:** same shape as GET.

### `PUT /api/v1/managers/{id}`

**Request body:** same shape as POST.

**Response 200:** same shape as GET.

### `DELETE /api/v1/managers/{id}`

**Response 204.** Returns 409 if this manager manages any house.

---

## Persons

### `GET /api/v1/persons`

**Response 200:**
```json
[
  { "id": 5, "first_name": "Anna", "last_name": "Schmidt", "email": "anna@example.com", "phone": "+49987654321" }
]
```

### `GET /api/v1/persons/{id}`

**Response 200:** single person object.

### `POST /api/v1/persons`

**Request body:**
```json
{ "first_name": "Anna", "last_name": "Schmidt", "email": "anna@example.com", "phone": "+49987654321" }
```

**Response 201:** same shape as GET.

### `PUT /api/v1/persons/{id}`

**Request body:** same shape as POST.

**Response 200:** same shape as GET.

### `DELETE /api/v1/persons/{id}`

**Response 204.** Returns 409 if this person has any booking.

---

## Houses

### `GET /api/v1/houses`

**Response 200:**
```json
[
  {
    "id": 42,
    "name": "Villa Rosa",
    "description": "Beachfront villa with pool",
    "address": {
      "id": 10,
      "street": "Calle del Mar",
      "number": "5",
      "postcode": "07001",
      "city": "Palma",
      "province": "Mallorca",
      "country": { "id": 2, "name": "Spain", "iso_code": "ES" }
    },
    "manager": { "id": 1, "first_name": "Hans", "last_name": "Müller", "email": "hans@example.com", "phone": "+49123456789" }
  }
]
```

### `GET /api/v1/houses/{id}`

**Response 200:** single house object (same shape as list item, plus `description`).

### `POST /api/v1/houses`

**Request body:**
```json
{
  "name": "Villa Rosa",
  "description": "Beachfront villa with pool",
  "address_id": 10,
  "manager_id": 1
}
```

**Response 201:** same shape as GET (with embedded address and manager).

### `PUT /api/v1/houses/{id}`

**Request body:** same shape as POST.

**Response 200:** same shape as GET.

### `DELETE /api/v1/houses/{id}`

**Response 204.** Returns 409 if this house has any booking.

---

## Calendar

Calendar entries are a sub-resource of a house. Status values: `NotRentable`, `Rentable`, `Rented`.
`Rented` is set only by the booking process — it cannot be set via calendar endpoints.

### `GET /api/v1/houses/{house_id}/calendar`

Optional query parameters `from` and `to` (both `YYYY-MM-DD`) filter by date range.

```
GET /api/v1/houses/42/calendar?from=2024-06-01&to=2024-06-30
```

**Response 200:**
```json
[
  { "id": 201, "date": "2024-06-01", "status": "Rentable", "price": 120.00 },
  { "id": 202, "date": "2024-06-02", "status": "Rented",   "price": 120.00 }
]
```

### `GET /api/v1/houses/{house_id}/calendar/{id}`

**Response 200:** single calendar entry (same shape as list item).

### `POST /api/v1/houses/{house_id}/calendar`

Creates one entry per day in the given range. Days that already have an entry are silently skipped.
`Rented` status is rejected with 422.

**Request body:**
```json
{
  "from": "2024-07-01",
  "to": "2024-07-31",
  "status": "Rentable",
  "price": 150.00
}
```

**Response 201:** array of the newly created entries (skipped days are not included).
```json
[
  { "id": 301, "date": "2024-07-01", "status": "Rentable", "price": 150.00 },
  { "id": 302, "date": "2024-07-02", "status": "Rentable", "price": 150.00 }
]
```

### `PATCH /api/v1/houses/{house_id}/calendar`

Updates the price of all existing entries in the given range. Does not affect booking payment records.

**Request body:**
```json
{
  "from": "2024-07-01",
  "to": "2024-07-31",
  "price": 175.00
}
```

**Response 200:** array of the updated entries.

### `DELETE /api/v1/houses/{house_id}/calendar?from=YYYY-MM-DD&to=YYYY-MM-DD`

Deletes all entries in the range. Returns 409 if any entry in the range is referenced by a
non-cancelled booking — the entire operation is rejected; no entries are deleted.

**Response 204.**

---

## Bookings

Booking status values: `Active`, `Cancelled`.

### `GET /api/v1/bookings`

Optional query parameters: `house_id`, `person_id`.

```
GET /api/v1/bookings?house_id=42
GET /api/v1/bookings?person_id=5
```

**Response 200:**
```json
[
  {
    "id": 101,
    "house": { "id": 42, "name": "Villa Rosa" },
    "person": { "id": 5, "first_name": "Anna", "last_name": "Schmidt" },
    "from": "2024-06-10",
    "to": "2024-06-20",
    "status": "Active",
    "paid_at": null,
    "total_paid": null
  }
]
```

### `GET /api/v1/bookings/{id}`

**Response 200:** full booking including embedded house and person, payment info.
```json
{
  "id": 101,
  "house": { "id": 42, "name": "Villa Rosa" },
  "person": { "id": 5, "first_name": "Anna", "last_name": "Schmidt" },
  "from": "2024-06-10",
  "to": "2024-06-20",
  "status": "Active",
  "paid_at": null,
  "total_paid": null
}
```

### `POST /api/v1/bookings`

All days in `from`–`to` must be in `Rentable` status; otherwise 422. On success, those calendar
entries are flipped to `Rented` internally. The `expected_total_price` is computed from the daily
prices at booking time and returned in the 201 response only — it is not persisted.

**Request body:**
```json
{
  "house_id": 42,
  "person_id": 5,
  "from": "2024-06-10",
  "to": "2024-06-20"
}
```

**Response 201:**
```json
{
  "id": 101,
  "house": { "id": 42, "name": "Villa Rosa" },
  "person": { "id": 5, "first_name": "Anna", "last_name": "Schmidt" },
  "from": "2024-06-10",
  "to": "2024-06-20",
  "status": "Active",
  "expected_total_price": 1320.00,
  "paid_at": null,
  "total_paid": null
}
```

### `POST /api/v1/bookings/{id}/cancel`

Cancels the booking. Calendar entries are reset to `Rentable`. Payment fields are cleared. Returns
422 if the booking is already cancelled.

**Request body:** none.

**Response 200:** updated booking (status `Cancelled`, `paid_at` null, `total_paid` null).

### `POST /api/v1/bookings/{id}/payment`

Records payment. Returns 422 if the booking is cancelled.

**Request body:**
```json
{
  "paid_at": "2024-05-20",
  "total_paid": 1320.00
}
```

**Response 200:** updated booking with `paid_at` and `total_paid` set.
