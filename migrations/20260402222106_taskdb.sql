-- taskchampion storage schema (equivalent to rusqlite schema v0.2)
CREATE TABLE IF NOT EXISTS taskdb_tasks (uuid TEXT PRIMARY KEY, data TEXT NOT NULL);

CREATE TABLE IF NOT EXISTS taskdb_operations (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  data TEXT NOT NULL,
  uuid TEXT GENERATED ALWAYS AS (
    coalesce(
      json_extract (data, '$.Update.uuid'),
      json_extract (data, '$.Create.uuid'),
      json_extract (data, '$.Delete.uuid')
    )
  ) VIRTUAL,
  synced BOOLEAN NOT NULL DEFAULT false
);

CREATE INDEX IF NOT EXISTS taskdb_operations_by_uuid ON taskdb_operations (uuid);

CREATE INDEX IF NOT EXISTS taskdb_operations_by_synced ON taskdb_operations (synced);

CREATE TABLE IF NOT EXISTS taskdb_working_set (id INTEGER PRIMARY KEY, uuid TEXT NOT NULL);

CREATE TABLE IF NOT EXISTS taskdb_sync_meta (key TEXT PRIMARY KEY, value TEXT NOT NULL);
