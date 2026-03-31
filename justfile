set dotenv-load := true
default:
  just --list

[group('rust')]
run:
  DATABASE_URL="$DB_URL" cargo run

[group('rust')]
build:
  DATABASE_URL="$DB_URL" cargo build

[group('rust')]
watch:
  DATABASE_URL="$DB_URL" cargo watch --ignore 'public/**' -x run

# run pending migrations
[group('db')]
migrate:
  DB_PATH=$(echo "$DB_URL" | sed 's|sqlite://||'); \
  mkdir -p "$(dirname "$DB_PATH")"; \
  [ -f "$DB_PATH" ] || touch "$DB_PATH"; \
  sqlx migrate run --database-url "$DB_URL"

# create a new empty migration file in migrations
[group('db')]
migrate-create name:
  sqlx migrate add {{name}} --source migrations

# revert the last applied migration (requires a .down.sql file)
[group('db')]
migrate-revert:
  sqlx migrate revert --database-url "$DB_URL"

# show which migrations have been applied and which are pending
[group('db')]
migrate-status:
  sqlx migrate info --database-url "$DB_URL"

# genearate sql types
[group('db')]
prepare:
  DATABASE_URL="$DB_URL" cargo sqlx prepare --database-url "$DB_URL"
