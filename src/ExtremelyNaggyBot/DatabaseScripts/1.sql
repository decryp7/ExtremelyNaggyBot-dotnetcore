CREATE TABLE IF NOT EXISTS users (
	user_id INTEGER PRIMARY KEY,
	first_name TEXT NOT NULL,
	last_name TEXT NOT NULL,
	timezone INTEGER NOT NULL
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

CREATE TABLE IF NOT EXISTS db_info (
	major_version INTEGER,
	minor_version INTEGER
);


