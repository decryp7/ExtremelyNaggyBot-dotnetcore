-- IANA timezone name (e.g. "Europe/London"); NULL means a fixed UTC offset is used.
ALTER TABLE users ADD COLUMN timezone TEXT;

-- Ids appear in inline buttons of old messages, so they must never be reused: rebuild the
-- tables with AUTOINCREMENT (without it SQLite may hand out the id of a deleted row again).
CREATE TABLE reminders_new (
    reminder_id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id     INTEGER NOT NULL REFERENCES users (user_id) ON DELETE CASCADE,
    description TEXT    NOT NULL,
    due_at      TEXT    NOT NULL,
    recurrence  INTEGER NOT NULL
);
INSERT INTO reminders_new SELECT reminder_id, user_id, description, due_at, recurrence FROM reminders;

CREATE TABLE naggings_new (
    nagging_id  INTEGER PRIMARY KEY AUTOINCREMENT,
    reminder_id INTEGER REFERENCES reminders (reminder_id) ON DELETE SET NULL,
    user_id     INTEGER NOT NULL REFERENCES users (user_id) ON DELETE CASCADE,
    description TEXT    NOT NULL,
    next_at     TEXT    NOT NULL
);
INSERT INTO naggings_new SELECT nagging_id, reminder_id, user_id, description, next_at FROM naggings;

DROP TABLE naggings;
DROP TABLE reminders;
ALTER TABLE reminders_new RENAME TO reminders;
ALTER TABLE naggings_new RENAME TO naggings;

CREATE INDEX reminders_due_at ON reminders (due_at);
CREATE INDEX reminders_user_id ON reminders (user_id);
CREATE INDEX naggings_next_at ON naggings (next_at);
CREATE INDEX naggings_reminder_id ON naggings (reminder_id);
CREATE INDEX naggings_user_id ON naggings (user_id);

-- Text a user sent that is waiting for a time to be picked. Not linked to users because
-- unregistered users can start a reminder before choosing their timezone.
CREATE TABLE drafts (
    draft_id   INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id    INTEGER NOT NULL,
    text       TEXT    NOT NULL,
    created_at TEXT    NOT NULL
);

CREATE INDEX drafts_user_id ON drafts (user_id);
