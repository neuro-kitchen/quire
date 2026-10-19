-- quire library schema, version 1.
-- Every record table carries created_at / updated_at (ms since epoch), updated_by (device ID)
-- and deleted_at (soft delete, kept as a tombstone for sync). IDs are 16-byte UUIDv7 BLOBs.

-- Facts about this library copy (e.g. `device_id`).
CREATE TABLE library_meta (
    key   TEXT PRIMARY KEY,
    value TEXT NOT NULL
) STRICT;

-- References. Bibliographic content is stored as JSON (`ReferenceData`), so new fields need no
-- migration; columns hold what is queried (cite key, kind, title, year).
CREATE TABLE reference (
    id          BLOB PRIMARY KEY,
    cite_key    TEXT NOT NULL UNIQUE,       -- unique also across the trash
    kind        TEXT NOT NULL,
    title       TEXT,
    year        INTEGER,
    data        TEXT NOT NULL,              -- ReferenceData as JSON
    provenance  TEXT NOT NULL,              -- field → Provenance as JSON
    created_at  INTEGER NOT NULL,
    updated_at  INTEGER NOT NULL,
    updated_by  BLOB NOT NULL,
    deleted_at  INTEGER
) STRICT;

-- Normalised identifiers: one work per (kind, key).
CREATE TABLE reference_identifier (
    kind          TEXT NOT NULL,            -- doi, pmid, pmcid, arxiv
    key           TEXT NOT NULL,            -- normalised (Identifier::key)
    value         TEXT NOT NULL,            -- as written
    reference_id  BLOB NOT NULL REFERENCES reference(id),
    PRIMARY KEY (kind, key)
) STRICT;
CREATE INDEX reference_identifier_reference ON reference_identifier(reference_id);

-- Records as received from a source, for re-deriving fields without fetching again.
CREATE TABLE raw_record (
    id            INTEGER PRIMARY KEY,
    reference_id  BLOB NOT NULL REFERENCES reference(id),
    source        TEXT NOT NULL,
    format        TEXT NOT NULL,
    body          TEXT NOT NULL,
    received_at   INTEGER NOT NULL
) STRICT;
CREATE INDEX raw_record_reference ON raw_record(reference_id);

-- Projects, nested through parent_id.
CREATE TABLE project (
    id          BLOB PRIMARY KEY,
    parent_id   BLOB REFERENCES project(id),
    name        TEXT NOT NULL,
    slug        TEXT NOT NULL,
    created_at  INTEGER NOT NULL,
    updated_at  INTEGER NOT NULL,
    updated_by  BLOB NOT NULL,
    deleted_at  INTEGER
) STRICT;
-- Slugs are unique among the live children of one parent (top level: parent_id NULL).
CREATE UNIQUE INDEX project_sibling_slug ON project(coalesce(parent_id, x''), slug) WHERE deleted_at IS NULL;

-- A reference in a project. Unlinking sets deleted_at (tombstone).
CREATE TABLE project_reference (
    project_id    BLOB NOT NULL REFERENCES project(id),
    reference_id  BLOB NOT NULL REFERENCES reference(id),
    created_at    INTEGER NOT NULL,
    updated_at    INTEGER NOT NULL,
    updated_by    BLOB NOT NULL,
    deleted_at    INTEGER,
    PRIMARY KEY (project_id, reference_id)
) STRICT;
CREATE INDEX project_reference_reference ON project_reference(reference_id);
