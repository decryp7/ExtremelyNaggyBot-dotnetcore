﻿CREATE TABLE IF NOT EXISTS users (
	user_id INTEGER PRIMARY KEY,
	first_name TEXT NOT NULL,
	last_name TEXT NOT NULL,
	timezone_offset INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS reminders (
	user_id INTEGER,
	description TEXT NOT NULL,
	datetime TEXT NOT NULL,
	recurring INTEGER NOT NULL,
	FOREIGN KEY (user_id)
		REFERENCES users (user_id)
			ON DELETE CASCADE
			ON UPDATE NO ACTION
);

CREATE TABLE IF NOT EXISTS naggings (
	reminder_id INTEGER,
	user_id INTEGER,
	description TEXT NOT NULL,
	datetime TEXT NOT NULL,
	FOREIGN KEY (reminder_id)
		REFERENCES reminders (row_id)
			ON DELETE CASCADE
			ON UPDATE NO ACTION
    FOREIGN KEY (user_id)
		REFERENCES users (user_id)
			ON DELETE CASCADE
			ON UPDATE NO ACTION
);

CREATE TABLE IF NOT EXISTS db_info (
	major_version INTEGER,
	minor_version INTEGER
);

INSERT INTO db_info values (1, 0);


