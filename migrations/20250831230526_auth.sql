-- Add migration script here
CREATE TABLE auth (
    user_id BLOB PRIMARY KEY NOT NULL,
    registration TEXT,        -- JSON for PasskeyRegistration
    authentication TEXT,      -- JSON for PasskeyAuthentication
    passkeys TEXT NOT NULL    -- JSON array for Vec<Passkey>
);
