-- Add migration script here
CREATE TABLE "users" (
  "id" bigint PRIMARY KEY,
  "pronouns" varchar,
  "lastfm" varchar
);

CREATE TABLE "muted" (
  "id" bigint PRIMARY KEY,
  "guild_id" bigint,
  "mute_length" int,
  "muted_time" timestamp,
  "unmute_time" timestamp
);

CREATE TABLE "guilds" (
  "id" bigint PRIMARY KEY,
  "muted_role" bigint
);

ALTER TABLE "muted" ADD FOREIGN KEY ("id") REFERENCES "users" ("id");

ALTER TABLE "muted" ADD FOREIGN KEY ("guild_id") REFERENCES "guilds" ("id");
