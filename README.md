# Extremely Naggy Bot

[![CI](https://dev.decryptology.net/decryp7/ExtremelyNaggyBot/actions/workflows/ci.yml/badge.svg?branch=master)](https://dev.decryptology.net/decryp7/ExtremelyNaggyBot/actions)

A Telegram reminder bot that does not let you forget: once a reminder fires it repeats every
minute until you press **Acknowledge**.

Written in Rust ([teloxide](https://github.com/teloxide/teloxide) + SQLite).

## Commands

| Command | Description |
| --- | --- |
| `/register <utc_offset>` | Register, or change your timezone. Examples: `+8`, `-5`, `+5:30`. |
| `/remindme <description> <dd/mm/yyyy> <HH:MM> [once\|daily\|weekly\|monthly]` | Add a reminder (24-hour time, `once` by default). |
| `/listreminders` | List your reminders. |
| `/removereminder <id>` | Remove a reminder and stop its nagging. |
| `/removeallreminders` | Remove all your reminders. |
| `/unregister` | Remove yourself and all your data. |
| `/help`, `/about` | Help and information. |

Example: `/remindme Take pills 25/12/2026 08:00 daily`

A recurring reminder that has not been acknowledged keeps nagging until it fires again. That
next occurrence then replaces the old nagging.

## Running with Docker Compose

```sh
cp .env.example .env               # then set TELEGRAM_BOT_TOKEN
mkdir -p data && sudo chown 10001:10001 data
docker compose up -d
docker compose logs -f
```

The SQLite database lives in `./data/extremelynaggybot.db`.

### Configuration

| Variable | Required | Description |
| --- | --- | --- |
| `TELEGRAM_BOT_TOKEN` | yes | Bot token from @BotFather. |
| `ADMIN_CHATID` | no | Chat that receives an "online" message at startup. |
| `SENTRY_DSN` | no | Enables Sentry error reporting. |
| `DATABASE_PATH` | no | Defaults to `data/extremelynaggybot.db`. |
| `RUST_LOG` | no | Log filter, defaults to `info`. |

### Upgrading from the .NET version

The existing database is upgraded automatically on first start. Users, reminders and
timezones are kept, and pending naggings are cleared. **Back up `data/` first**: the
upgraded database cannot be read by the old version.

The container now runs as a non-root user (uid 10001). Run
`sudo chown -R 10001:10001 data` on the existing data directory.

## Development

```sh
cargo test
cargo clippy --all-targets -- -D warnings
cargo fmt
TELEGRAM_BOT_TOKEN=... cargo run
```

CI (`.gitea/workflows/ci.yml`) runs format, clippy and tests on every push and pull request.
Pushes to `master` also build and push `docker.decryptology.net/decryptology/extremelynaggybot`.
The workflow needs the `REPO_USERNAME` and `REPO_PASSWORD` repository secrets.

## Source

https://dev.decryptology.net/decryp7/ExtremelyNaggyBot — bug reports: @decryp7
