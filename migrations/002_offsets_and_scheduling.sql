-- Rebuilds the schema created by the original .NET bot:
--   * users: timezone offset is stored in minutes so half-hour zones (e.g. +05:30) work.
--   * reminders: `due_at` is the next time the reminder fires; orphaned rows are dropped.
--   * naggings: no longer deleted when their (one-off) reminder is consumed, and keyed
--     by their own id so acknowledgements target a single nagging.
-- Existing naggings are discarded; the old bot cleared them on every start anyway.

CREATE TABLE users_new (
    user_id            INTEGER PRIMARY KEY,
    first_name         TEXT    NOT NULL,
    last_name          TEXT    NOT NULL DEFAULT '',
    utc_offset_minutes INTEGER NOT NULL
);

INSERT INTO users_new (user_id, first_name, last_name, utc_offset_minutes)
SELECT user_id, first_name, last_name, timezone_offset * 60 FROM users;

CREATE TABLE reminders_new (
    reminder_id INTEGER PRIMARY KEY,
    user_id     INTEGER NOT NULL REFERENCES users (user_id) ON DELETE CASCADE,
    description TEXT    NOT NULL,
    due_at      TEXT    NOT NULL,
    recurrence  INTEGER NOT NULL
);

INSERT INTO reminders_new (reminder_id, user_id, description, due_at, recurrence)
SELECT reminder_id, user_id, description, datetime, recurring
FROM reminders
WHERE user_id IN (SELECT user_id FROM users);

CREATE TABLE naggings_new (
    nagging_id  INTEGER PRIMARY KEY,
    reminder_id INTEGER REFERENCES reminders (reminder_id) ON DELETE SET NULL,
    user_id     INTEGER NOT NULL REFERENCES users (user_id) ON DELETE CASCADE,
    description TEXT    NOT NULL,
    next_at     TEXT    NOT NULL
);

DROP TABLE naggings;
DROP TABLE reminders;
DROP TABLE users;
DROP TABLE IF EXISTS db_info;

ALTER TABLE users_new RENAME TO users;
ALTER TABLE reminders_new RENAME TO reminders;
ALTER TABLE naggings_new RENAME TO naggings;

CREATE INDEX reminders_due_at ON reminders (due_at);
CREATE INDEX reminders_user_id ON reminders (user_id);
CREATE INDEX naggings_next_at ON naggings (next_at);
CREATE INDEX naggings_reminder_id ON naggings (reminder_id);
CREATE INDEX naggings_user_id ON naggings (user_id);
