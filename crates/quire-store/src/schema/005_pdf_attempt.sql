-- quire library schema, version 5: the last `pdf get` attempt for references without a PDF, so
-- lists can tell "not found" and "blocked" from "never tried". Local state (not synced).

CREATE TABLE pdf_attempt (
    reference_id  BLOB PRIMARY KEY REFERENCES reference(id),
    outcome       TEXT NOT NULL,            -- blocked, not-found, no-identifier, error
    detail        TEXT,
    attempted_at  INTEGER NOT NULL
) STRICT;
