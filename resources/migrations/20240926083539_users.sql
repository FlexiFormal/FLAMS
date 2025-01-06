-- Add migration script here
CREATE TABLE IF NOT EXISTS users (
  id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
  gitlab_id INTEGER UNIQUE NOT NULL,
  name TEXT NOT NULL,
  username VARCHAR(255) UNIQUE NOT NULL,
  email TEXT NOT NULL,
  avatar_url TEXT NOT NULL,
  can_create_group BOOLEAN NOT NULL,
  can_create_project BOOLEAN NOT NULL,
  secret VARCHAR(255) NOT NULL,
  secret_hash BLOB NOT NULL,
  is_admin BOOLEAN NOT NULL
);