-- quire library schema, version 3: automatic exports ("keep updated", as Better BibTeX's
-- auto-export). After a change, each registered file is rewritten if its content changed.

CREATE TABLE auto_export (
    path          TEXT PRIMARY KEY,         -- absolute path of the written file
    project_id    BLOB REFERENCES project(id),  -- NULL: the whole library
    recursive     INTEGER NOT NULL,         -- 0/1: include subprojects
    profile       TEXT,                     -- export profile name in the config, if any
    format        TEXT,                     -- format overriding the profile's, if any
    created_at    INTEGER NOT NULL,
    written_at    INTEGER                   -- last time the file was written
) STRICT;
