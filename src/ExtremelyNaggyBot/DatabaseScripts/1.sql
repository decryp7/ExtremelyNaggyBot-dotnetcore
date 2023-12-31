﻿CREATE TABLE IF NOT EXISTS users (
	user_id INTEGER PRIMARY KEY,
	first_name TEXT NOT NULL,
	last_name TEXT NOT NULL,
	timezone_offset INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS user_index
	ON users (user_id);

CREATE TABLE IF NOT EXISTS reminders (
	reminder_id INTEGER PRIMARY KEY,
	user_id INTEGER,
	description TEXT NOT NULL,
	datetime TEXT NOT NULL,
	recurring INTEGER NOT NULL,
	FOREIGN KEY (user_id)
		REFERENCES users (user_id)
			ON DELETE CASCADE
			ON UPDATE NO ACTION
);

CREATE INDEX IF NOT EXISTS reminder_index
	ON reminders (reminder_id);

CREATE TABLE IF NOT EXISTS naggings (
	nagging_id INTEGER PRIMARY KEY,
	reminder_id INTEGER,
	user_id INTEGER,
	description TEXT NOT NULL,
	datetime TEXT NOT NULL,
	FOREIGN KEY (reminder_id)
		REFERENCES reminders (reminder_id)
			ON DELETE CASCADE
			ON UPDATE NO ACTION
    FOREIGN KEY (user_id)
		REFERENCES users (user_id)
			ON DELETE CASCADE
			ON UPDATE NO ACTION
);

CREATE INDEX IF NOT EXISTS nagging_index
	ON naggings (nagging_id);

CREATE TABLE IF NOT EXISTS db_info (
	major_version INTEGER,
	minor_version INTEGER
);

REPLACE INTO db_info (rowid, major_version, minor_version) values (1, 1, 0);


