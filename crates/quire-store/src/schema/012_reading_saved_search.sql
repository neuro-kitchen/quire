-- Personal records: where the user is with each paper, and searches kept by name. Synced to the
-- user's own devices (change log below); never to collaborators (`.knowledge/sync.md`).

CREATE TABLE reading (
    reference_id  BLOB PRIMARY KEY REFERENCES reference(id),
    status        TEXT NOT NULL,
    started_at    INTEGER,
    finished_at   INTEGER,
    created_at    INTEGER NOT NULL,
    updated_at    INTEGER NOT NULL,
    updated_by    BLOB NOT NULL,
    deleted_at    INTEGER
) STRICT;
CREATE INDEX reading_status ON reading(status);

CREATE TABLE saved_search (
    id          BLOB PRIMARY KEY,
    name        TEXT NOT NULL,
    key         TEXT NOT NULL,
    query       TEXT NOT NULL,
    created_at  INTEGER NOT NULL,
    updated_at  INTEGER NOT NULL,
    updated_by  BLOB NOT NULL,
    deleted_at  INTEGER
) STRICT;
-- One live search per name (ignoring case).
CREATE UNIQUE INDEX saved_search_key ON saved_search(key) WHERE deleted_at IS NULL;

CREATE TRIGGER change_reading_insert AFTER INSERT ON reading BEGIN
    INSERT OR REPLACE INTO change_log (tbl, k1, k2, origin)
    VALUES ('reading', NEW.reference_id, x'', (SELECT value FROM sync_state WHERE key = 'merging_from'));
END;
CREATE TRIGGER change_reading_update AFTER UPDATE ON reading BEGIN
    INSERT OR REPLACE INTO change_log (tbl, k1, k2, origin)
    VALUES ('reading', NEW.reference_id, x'', (SELECT value FROM sync_state WHERE key = 'merging_from'));
END;
CREATE TRIGGER change_saved_search_insert AFTER INSERT ON saved_search BEGIN
    INSERT OR REPLACE INTO change_log (tbl, k1, k2, origin)
    VALUES ('saved_search', NEW.id, x'', (SELECT value FROM sync_state WHERE key = 'merging_from'));
END;
CREATE TRIGGER change_saved_search_update AFTER UPDATE ON saved_search BEGIN
    INSERT OR REPLACE INTO change_log (tbl, k1, k2, origin)
    VALUES ('saved_search', NEW.id, x'', (SELECT value FROM sync_state WHERE key = 'merging_from'));
END;
