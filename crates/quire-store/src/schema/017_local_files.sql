-- Lazy PDFs (`.knowledge/sync-hub.md`): what this device knows about its PDF files, kept on this
-- device only (no change-log triggers, never synced). A file is evicted only when the hub
-- confirmed it holds it (`on_hub`), never when it may be the last copy; marked papers and
-- projects stay downloaded.
CREATE TABLE file_local (
    hash         TEXT PRIMARY KEY,        -- BLAKE3, hex
    on_hub       INTEGER NOT NULL DEFAULT 0,
    last_opened  INTEGER                  -- milliseconds since 1970
) STRICT;

CREATE TABLE keep_offline (
    kind    TEXT NOT NULL CHECK (kind IN ('reference', 'project')),
    target  BLOB NOT NULL,                -- the reference's or project's ID
    PRIMARY KEY (kind, target)
) STRICT;
