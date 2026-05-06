-- =============================================================================
-- 0001 — initial schema
-- =============================================================================

-- Countries ----------------------------------------------------------------
CREATE TABLE countries (
    id       BIGSERIAL   PRIMARY KEY,
    name     VARCHAR(100) NOT NULL,
    iso_code CHAR(2)      NOT NULL,
    CONSTRAINT uq_countries_iso_code UNIQUE (iso_code),
    CONSTRAINT ck_countries_iso_code CHECK (iso_code ~ '^[A-Z]{2}$')
);

-- Addresses ----------------------------------------------------------------
CREATE TABLE addresses (
    id         BIGSERIAL    PRIMARY KEY,
    street     VARCHAR(200) NOT NULL,
    number     VARCHAR(20)  NOT NULL,
    postcode   VARCHAR(20)  NOT NULL,
    city       VARCHAR(100) NOT NULL,
    province   VARCHAR(100),
    country_id BIGINT       NOT NULL REFERENCES countries (id)
);

-- Managers -----------------------------------------------------------------
CREATE TABLE managers (
    id         BIGSERIAL    PRIMARY KEY,
    first_name VARCHAR(100) NOT NULL,
    last_name  VARCHAR(100) NOT NULL,
    email      VARCHAR(255) NOT NULL,
    phone      VARCHAR(50)  NOT NULL,
    CONSTRAINT uq_managers_email UNIQUE (email)
);

-- Persons ------------------------------------------------------------------
CREATE TABLE persons (
    id         BIGSERIAL    PRIMARY KEY,
    first_name VARCHAR(100) NOT NULL,
    last_name  VARCHAR(100) NOT NULL,
    email      VARCHAR(255) NOT NULL,
    phone      VARCHAR(50)  NOT NULL,
    CONSTRAINT uq_persons_email UNIQUE (email)
);

-- Houses -------------------------------------------------------------------
CREATE TABLE houses (
    id          BIGSERIAL    PRIMARY KEY,
    name        VARCHAR(200) NOT NULL,
    description TEXT         NOT NULL,
    address_id  BIGINT       NOT NULL REFERENCES addresses (id),
    manager_id  BIGINT       NOT NULL REFERENCES managers  (id)
);

-- Calendar entries (one row per house per day) ----------------------------
CREATE TABLE calendar (
    id       BIGSERIAL      PRIMARY KEY,
    house_id BIGINT         NOT NULL REFERENCES houses (id),
    date     DATE           NOT NULL,
    status   VARCHAR(20)    NOT NULL DEFAULT 'NotRentable',
    price    NUMERIC(10, 2) NOT NULL,
    CONSTRAINT uq_calendar_house_date UNIQUE (house_id, date),
    CONSTRAINT ck_calendar_status     CHECK  (status IN ('NotRentable', 'Rentable', 'Rented'))
);

-- Bookings ----------------------------------------------------------------
CREATE TABLE bookings (
    id               BIGSERIAL      PRIMARY KEY,
    house_id         BIGINT         NOT NULL REFERENCES houses    (id),
    person_id        BIGINT         NOT NULL REFERENCES persons   (id),
    from_calendar_id BIGINT         NOT NULL REFERENCES calendar  (id),
    to_calendar_id   BIGINT         NOT NULL REFERENCES calendar  (id),
    status           VARCHAR(20)    NOT NULL DEFAULT 'Active',
    paid_at          DATE,
    total_paid       NUMERIC(10, 2),
    CONSTRAINT ck_bookings_status  CHECK (status IN ('Active', 'Cancelled')),
    -- paid_at and total_paid must both be set or both be null
    CONSTRAINT ck_bookings_payment CHECK (
        (paid_at IS NULL) = (total_paid IS NULL)
    )
);

-- Indexes -----------------------------------------------------------------
CREATE INDEX ix_addresses_country_id   ON addresses (country_id);
CREATE INDEX ix_houses_address_id      ON houses    (address_id);
CREATE INDEX ix_houses_manager_id      ON houses    (manager_id);
CREATE INDEX ix_calendar_house_date    ON calendar  (house_id, date);
CREATE INDEX ix_bookings_house_id      ON bookings  (house_id);
CREATE INDEX ix_bookings_person_id     ON bookings  (person_id);
CREATE INDEX ix_bookings_from_calendar ON bookings  (from_calendar_id);
CREATE INDEX ix_bookings_to_calendar   ON bookings  (to_calendar_id);
