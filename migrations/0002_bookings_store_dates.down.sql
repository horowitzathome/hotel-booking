-- Revert 0002: restore original schema (NOT NULL FK columns, no stored dates).
ALTER TABLE bookings
    DROP COLUMN from_date,
    DROP COLUMN to_date;

ALTER TABLE bookings
    ALTER COLUMN from_calendar_id SET NOT NULL,
    ALTER COLUMN to_calendar_id   SET NOT NULL;

ALTER TABLE bookings DROP CONSTRAINT bookings_from_calendar_id_fkey;
ALTER TABLE bookings DROP CONSTRAINT bookings_to_calendar_id_fkey;

ALTER TABLE bookings
    ADD CONSTRAINT bookings_from_calendar_id_fkey
        FOREIGN KEY (from_calendar_id) REFERENCES calendar (id);

ALTER TABLE bookings
    ADD CONSTRAINT bookings_to_calendar_id_fkey
        FOREIGN KEY (to_calendar_id) REFERENCES calendar (id);
