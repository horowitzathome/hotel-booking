---
name: project-context
description: What hotel-booking is, its audience, and its educational purpose
metadata:
  type: project
---

hotel-booking is a fully built Rust REST API (actix-web + SQLx + Postgres) for managing house rentals — countries, addresses, managers, persons, houses, calendar, bookings. All 21 implementation steps are complete (Steps 1–21 per docs/implementation_plan.md).

**Why:** Educational demo showing Java/Spring Boot developers how idiomatic Rust works. Architecture mirrors Spring layers: handlers = @RestController, services = @Service, repositories = @Repository. docs/architecture.md has the full Spring/Rust comparison table.

**How to apply:** When explaining code or architecture, frame explanations in terms of Spring Boot equivalents. The target audience is experienced Java developers, not Rust beginners.
