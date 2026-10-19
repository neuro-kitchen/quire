-- Passages of a paper's text highlighted in the reader, anchored by the quote and the text
-- before it (not by position: the text can be extracted again or come from another source).
CREATE TABLE highlight (
    id            BLOB PRIMARY KEY,
    reference_id  BLOB NOT NULL REFERENCES reference(id),
    quote         TEXT NOT NULL,
    prefix        TEXT NOT NULL DEFAULT '',
    comment       TEXT,
    created_at    INTEGER NOT NULL,
    updated_at    INTEGER NOT NULL,
    updated_by    BLOB NOT NULL,
    deleted_at    INTEGER
) STRICT;
CREATE INDEX highlight_reference ON highlight(reference_id);
