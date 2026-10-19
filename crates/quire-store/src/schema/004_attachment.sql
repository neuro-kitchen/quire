-- quire library schema, version 4: attachments. The file is stored once in the library's
-- `files/` folder under the BLAKE3 hash of its content; a row says which reference it belongs to.

CREATE TABLE attachment (
    id            BLOB PRIMARY KEY,
    reference_id  BLOB NOT NULL REFERENCES reference(id),
    hash          TEXT NOT NULL,            -- BLAKE3, hex
    filename      TEXT NOT NULL,            -- readable name, e.g. vaswaniAttentionAllYou2017.pdf
    content_type  TEXT NOT NULL,
    size          INTEGER NOT NULL,
    source        TEXT NOT NULL,            -- URL, `browser`, or `file:<name>`
    pdf_kind      TEXT,                     -- text, scanned, mixed
    pages         INTEGER,
    created_at    INTEGER NOT NULL,
    updated_at    INTEGER NOT NULL,
    updated_by    BLOB NOT NULL,
    deleted_at    INTEGER
) STRICT;
CREATE INDEX attachment_reference ON attachment(reference_id);
-- One live row per file and reference.
CREATE UNIQUE INDEX attachment_reference_hash ON attachment(reference_id, hash) WHERE deleted_at IS NULL;
