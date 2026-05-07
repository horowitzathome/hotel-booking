-- =============================================================================
-- 0001 — rollback initial schema
-- Drop in reverse dependency order.
-- =============================================================================

DROP TABLE IF EXISTS bookings;
DROP TABLE IF EXISTS calendar;
DROP TABLE IF EXISTS houses;
DROP TABLE IF EXISTS persons;
DROP TABLE IF EXISTS managers;
DROP TABLE IF EXISTS addresses;
DROP TABLE IF EXISTS countries;

DROP TYPE IF EXISTS booking_status;
DROP TYPE IF EXISTS calendar_status;
