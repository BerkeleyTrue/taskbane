-- Add migration script here
ALTER table auth ADD COLUMN authorize_token TEXT;
ALTER table auth ADD COLUMN authorized TEXT NOT NULL;
