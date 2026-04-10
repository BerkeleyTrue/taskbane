set dotenv-load := true
default:
  just --list

[group('flake')]
flake-build:
  nix build

[group('flake')]
flake-update:
  nix flake update

[group('flake')]
flake-update-input input:
  nix flake update {{input}}

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

# bootstrap task db replica and push to sync server (task-sync-server must be running)
[group('task-sync')]
task-sync-init:
  mkdir -p "$(dirname "$TASK_DB_PATH")"
  cp ~/.config/task/taskchampion.sqlite3 "$TASK_DB_PATH"
  sqlite3 "$TASK_DB_PATH" "UPDATE sync_meta SET value = '00000000-0000-0000-0000-000000000000' WHERE key = 'base_version'; UPDATE operations SET synced = 0;"
  TASKDATA="$(dirname "$TASK_DB_PATH")" task rc.sync.server.url="$TASK_URL" rc.sync.server.client_id="$TASK_CLIENT_ID" rc.sync.encryption_secret="$TASK_SECRET" sync

# add a task to the local replica and sync to remote server
[group('task-sync')]
task-add description:
  TASKDATA="$(dirname "$TASK_DB_PATH")" task add "{{description}}"
  TASKDATA="$(dirname "$TASK_DB_PATH")" task rc.sync.server.url="$TASK_URL" rc.sync.server.client_id="$TASK_CLIENT_ID" rc.sync.encryption_secret="$TASK_SECRET" sync

# launch local taskchampion sync server for the app to connect to
[group('task-sync')]
task-sync-server:
  taskchampion-sync-server --data-dir "$TASK_SYNC_DIR" --listen "0.0.0.0:$TASK_SYNC_PORT"
