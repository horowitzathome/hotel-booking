# This file contains general infors for developers

## SQLx

SQLx also needs sqlx-cli installed once: cargo install sqlx-cli --no-default-features --features rustls,postgres

## DB Migration

just db-migrate-add name=add_booking_notes   # creates 0002_add_booking_notes.up.sql + .down.sql

just db-migrate # applies it

just db-migrate-revert # rolls it back if needed