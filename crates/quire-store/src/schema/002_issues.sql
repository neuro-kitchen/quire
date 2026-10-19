-- quire library schema, version 2: issues recorded when a reference was fetched, and inputs whose
-- lookup failed. Both are local state about this copy's lookups (not synced).

-- One row per reference and issue code (e.g. `fallback`, `blocked`); lints are not stored.
CREATE TABLE reference_issue (
    reference_id  BLOB NOT NULL REFERENCES reference(id),
    code          TEXT NOT NULL,
    detail        TEXT,
    recorded_at   INTEGER NOT NULL,
    PRIMARY KEY (reference_id, code)
) STRICT;

-- What `quire add` could not look up; removed once it is added.
CREATE TABLE failed_input (
    input       TEXT PRIMARY KEY,
    project_id  BLOB REFERENCES project(id),
    error       TEXT NOT NULL,
    attempts    INTEGER NOT NULL,
    first_at    INTEGER NOT NULL,
    last_at     INTEGER NOT NULL
) STRICT;
