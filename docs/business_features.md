# Business Features

This is a program for managing houses (villas) for rent to persons.

---

## Countries

- **List countries** — retrieve all countries with their ISO code
- **Get country** — retrieve a single country by ID
- **Create country** — add a new country (name + ISO code)
- **Update country** — change name or ISO code of an existing country
- **Delete country** — remove a country (only if no address references it)

---

## Addresses

- **Get address** — retrieve an address by ID (includes country)
- **Create address** — create a new address (street, number, postcode, city, optional province, country)
- **Update address** — update fields of an existing address
- **Delete address** — remove an address (only if no house references it)

---

## Managers

- **List managers** — retrieve all managers
- **Get manager** — retrieve a single manager by ID
- **Create manager** — register a new manager (first name, last name, email, phone)
- **Update manager** — update manager details
- **Delete manager** — remove a manager (only if they manage no houses)

---

## Houses

- **List houses** — retrieve all houses (with manager and address)
- **Get house** — retrieve a single house by ID (full detail: name, address, description, manager)
- **Create house** — register a new house with name, address, description, and responsible manager
- **Update house** — change name, description, address, or manager of a house
- **Delete house** — remove a house (only if it has no bookings)

---

## Calendar (per house)

The calendar is a set of dated entries per house that tracks availability and pricing for individual days.

- **List calendar entries for a house** — retrieve all calendar entries for a given house, optionally filtered by date range
- **Get calendar entry** — retrieve a single calendar entry by ID
- **Create calendar entries** — add day entries for a house with status (`NotRentable` or `Rentable`) and price for date period
  - `Rented` status is not allowed here; it is only set by the booking process
  - Ignore entries which already exist (i.e. for those do not create an entry)
- **Update price for calendar entries** — change price of existing calendar entries for a date period
  - Updates do not reflect any existing bookings, the real price a person paid is always stored in a booking
- **Delete calendar entries** — remove calendar entries for a date period (only if not referenced by a not cancelled booking)
  - If at least one entry is referenced by a not cancelled booking, the whole operation fails

---

## Persons (Renters)

- **List persons** — retrieve all persons
- **Get person** — retrieve a single person by ID
- **Create person** — register a new person (first name, last name, email, phone)
- **Update person** — update personal details
- **Delete person** — remove a person (only if they have no bookings)

---

## Bookings

- **List bookings** — retrieve all bookings, optionally filtered by house ID or person ID as query parameters
- **Get booking** — retrieve a single booking by ID (includes house, person, from/to calendar entries, payment info)
- **Create booking** — book a house for a person over a date range:
  - All days in the range must be in status `Rentable`
  - All days in the range are flipped to `Rented` (calendar status is managed internally by booking logic)
  - The expected total price is calculated from the daily prices at booking time but is **not persisted**; only the actual paid price is stored when payment is recorded
- **Cancel booking** — cancel an existing booking:
  - All calendar entries of the booking are reset to `Rentable` (managed internally by booking logic)
  - Payment fields are cleared
  - Booking is set to cancelled
- **Record payment** — mark a booking as paid by setting the payment date and total price paid

---

## Health & Observability

- **Health check** (`GET /health`) — returns service status; used by Kubernetes liveness and readiness probes
- **Metrics** (`GET /metrics`) — exposes Prometheus-format metrics for Grafana scraping
