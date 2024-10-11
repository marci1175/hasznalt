CREATE TABLE authorized_users (
  client_signature VARCHAR NOT NULL,
  session_id VARCHAR PRIMARY KEY NOT NULL,
  account_hash VARCHAR NOT NULL
)