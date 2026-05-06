-- =============================================================================
-- 0002 — store from/to dates directly on bookings so that cancelled bookings
--        do not block deletion of their calendar entries (FK constraint fix).
-- =============================================================================

-- Add stored date columns (nullable initially so we can back-fill).
ALTER TABLE bookings
    ADD COLUMN from_date DATE,
    ADD COLUMN to_date   DATE;

-- Back-fill from the calendar FK references.
UPDATE bookings b
SET from_date = c_from.date,
    to_date   = c_to.date
FROM calendar c_from, calendar c_to
WHERE c_from.id = b.from_calendar_id
  AND c_to.id   = b.to_calendar_id;

-- Now enforce NOT NULL (all rows are populated).
ALTER TABLE bookings
    ALTER COLUMN from_date SET NOT NULL,
    ALTER COLUMN to_date   SET NOT NULL;

-- Make the calendar FK columns nullable so that deleting calendar entries
-- belonging to a cancelled booking does not violate referential integrity.
ALTER TABLE bookings
    ALTER COLUMN from_calendar_id DROP NOT NULL,
    ALTER COLUMN to_calendar_id   DROP NOT NULL;

-- Replace the FKs with ON DELETE SET NULL so Postgres handles nullification
-- automatically when a referenced calendar row is deleted.
ALTER TABLE bookings DROP CONSTRAINT bookings_from_calendar_id_fkey;
ALTER TABLE bookings DROP CONSTRAINT bookings_to_calendar_id_fkey;

ALTER TABLE bookings
    ADD CONSTRAINT bookings_from_calendar_id_fkey
        FOREIGN KEY (from_calendar_id) REFERENCES calendar (id) ON DELETE SET NULL;

ALTER TABLE bookings
    ADD CONSTRAINT bookings_to_calendar_id_fkey
        FOREIGN KEY (to_calendar_id) REFERENCES calendar (id) ON DELETE SET NULL;
