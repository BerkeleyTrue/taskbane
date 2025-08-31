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
  DATABASE_URL="$DB_URL" cargo watch -x run

[group('db')]
migrate:
  sqlx migrate run --database-url "$DB_URL"

[group('db')]
migrate-create name:
  sqlx migrate add {{name}} --source migrations

[group('db')]
migrate-revert:
  sqlx migrate revert --database-url "$DB_URL"

[group('db')]
migrate-status:
  sqlx migrate info --database-url "$DB_URL"

[group('db')]
prepare:
  DATABASE_URL="$DB_URL" cargo sqlx prepare --database-url "$DB_URL"
