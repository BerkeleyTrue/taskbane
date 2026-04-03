-- Replace virtual generated uuid column with a real stored column.
-- The virtual column used json_extract which returns TEXT, causing type mismatches
-- when comparing against sqlx-bound UUIDs (stored as BLOB). The real column is
-- populated by Rust via sqlx so both sides of any comparison use the same type.
DROP INDEX IF EXISTS taskdb_operations_by_uuid;
ALTER TABLE taskdb_operations DROP COLUMN uuid;
ALTER TABLE taskdb_operations ADD COLUMN uuid BLOB;
CREATE INDEX IF NOT EXISTS taskdb_operations_by_uuid ON taskdb_operations (uuid);
