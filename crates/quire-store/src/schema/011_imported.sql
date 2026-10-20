-- Records imported from another reference manager (Zotero), by that program's key, so importing
-- again finds them (items without a DOI included). Kept on this device; not synced.
CREATE TABLE imported (
    source        TEXT NOT NULL,
    key           TEXT NOT NULL,
    reference_id  BLOB NOT NULL REFERENCES reference (id),
    imported_at   INTEGER NOT NULL,
    PRIMARY KEY (source, key)
) STRICT;
