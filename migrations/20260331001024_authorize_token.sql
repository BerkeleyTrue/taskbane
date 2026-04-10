-- Add migration script here
ALTER table auth ADD COLUMN authorize_token BLOB;
ALTER table auth ADD COLUMN authorized TEXT NOT NULL;
