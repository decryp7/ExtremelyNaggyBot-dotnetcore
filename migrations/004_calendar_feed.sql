-- Secret token in each user's calendar feed URL. NULL until the user asks for the feed;
-- replacing it revokes the old URL.
ALTER TABLE users ADD COLUMN calendar_token TEXT;

CREATE UNIQUE INDEX users_calendar_token ON users (calendar_token);
