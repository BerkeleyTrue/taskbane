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

# copy local taskwarrior db into project data dir for testing
[group('task-sync-server')]
task-db-copy:
  mkdir -p "$(dirname "$TASK_DB_PATH")"
  cp ~/.config/task/taskchampion.sqlite3 "$TASK_DB_PATH"
  echo "Copied taskchampion db to $TASK_DB_PATH"

# launch local taskchampion sync server
[group('task-sync-server')]
task-sync-server:
  mkdir -p "$TASK_SYNC_DIR"
  taskchampion-sync-server --data-dir "$TASK_SYNC_DIR" --listen "0.0.0.0:$TASK_SYNC_PORT"
