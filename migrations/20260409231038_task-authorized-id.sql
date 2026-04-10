-- Add migration script here
ALTER table auth ADD COLUMN authorized_task_id BLOB;
