-- quire library schema, version 6: Zotero-style tags, notes and related references. All carry
-- record times, device and soft delete, like references (tombstones for sync).

-- Labels; `key` is the lowercase name (Unicode, computed by quire), unique among live tags.
CREATE TABLE tag (
    id          BLOB PRIMARY KEY,
    name        TEXT NOT NULL,
    key         TEXT NOT NULL,
    color       TEXT,                       -- #rrggbb for colored tags
    position    INTEGER,                    -- 1–9 for colored tags (their key)
    created_at  INTEGER NOT NULL,
    updated_at  INTEGER NOT NULL,
    updated_by  BLOB NOT NULL,
    deleted_at  INTEGER
) STRICT;
CREATE UNIQUE INDEX tag_live_key ON tag(key) WHERE deleted_at IS NULL;
CREATE UNIQUE INDEX tag_live_position ON tag(position) WHERE position IS NOT NULL AND deleted_at IS NULL;

-- A tag on a reference; removing it keeps the row as a tombstone.
CREATE TABLE reference_tag (
    reference_id  BLOB NOT NULL REFERENCES reference(id),
    tag_id        BLOB NOT NULL REFERENCES tag(id),
    kind          TEXT NOT NULL,            -- manual, automatic
    created_at    INTEGER NOT NULL,
    updated_at    INTEGER NOT NULL,
    updated_by    BLOB NOT NULL,
    deleted_at    INTEGER,
    PRIMARY KEY (reference_id, tag_id)
) STRICT;
CREATE INDEX reference_tag_tag ON reference_tag(tag_id);

-- Markdown notes, on a reference or standalone (reference_id NULL).
CREATE TABLE note (
    id            BLOB PRIMARY KEY,
    reference_id  BLOB REFERENCES reference(id),
    body          TEXT NOT NULL,
    created_at    INTEGER NOT NULL,
    updated_at    INTEGER NOT NULL,
    updated_by    BLOB NOT NULL,
    deleted_at    INTEGER
) STRICT;
CREATE INDEX note_reference ON note(reference_id);

-- Related references, both ways; stored once with a < b (by ID bytes).
CREATE TABLE reference_relation (
    a           BLOB NOT NULL REFERENCES reference(id),
    b           BLOB NOT NULL REFERENCES reference(id),
    created_at  INTEGER NOT NULL,
    updated_at  INTEGER NOT NULL,
    updated_by  BLOB NOT NULL,
    deleted_at  INTEGER,
    PRIMARY KEY (a, b),
    CHECK (a < b)
) STRICT;
CREATE INDEX reference_relation_b ON reference_relation(b);
