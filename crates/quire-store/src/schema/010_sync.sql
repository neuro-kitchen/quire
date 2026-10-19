-- Sync (`.knowledge/sync.md`). Devices exchange the current state of records that changed;
-- `change_log` says which rows changed, numbered in this copy's own order (`seq`). A row changed
-- again moves to a new number (one entry per row). Triggers fill it, so no write path can forget.

-- This library's identity, shared by every device that syncs it (a new, empty copy adopts the
-- other's on its first sync).
INSERT OR IGNORE INTO library_meta (key, value) VALUES ('library_id', lower(hex(randomblob(16))));

CREATE TABLE change_log (
    seq     INTEGER PRIMARY KEY AUTOINCREMENT,
    tbl     TEXT NOT NULL,
    k1      BLOB NOT NULL,
    k2      BLOB NOT NULL DEFAULT x'',
    -- The device the change was merged from (NULL: made here); not sent back to it.
    origin  TEXT,
    UNIQUE (tbl, k1, k2)
) STRICT;

-- While merging another device's rows: which device (read by the triggers below).
CREATE TABLE sync_state (
    key    TEXT PRIMARY KEY,
    value  TEXT NOT NULL
) STRICT;

-- For each device synced with: the highest `seq` of theirs received.
CREATE TABLE sync_peer (
    device     TEXT PRIMARY KEY,
    received   INTEGER NOT NULL DEFAULT 0,
    synced_at  INTEGER
) STRICT;

-- The same project or tag created on two devices (same name, other ID): their ID → ours.
CREATE TABLE sync_alias (
    remote  BLOB PRIMARY KEY,
    local   BLOB NOT NULL
) STRICT;

CREATE TRIGGER change_reference_insert AFTER INSERT ON reference BEGIN
    INSERT OR REPLACE INTO change_log (tbl, k1, k2, origin)
    VALUES ('reference', NEW.id, x'', (SELECT value FROM sync_state WHERE key = 'merging_from'));
END;
CREATE TRIGGER change_reference_update AFTER UPDATE ON reference BEGIN
    INSERT OR REPLACE INTO change_log (tbl, k1, k2, origin)
    VALUES ('reference', NEW.id, x'', (SELECT value FROM sync_state WHERE key = 'merging_from'));
END;
CREATE TRIGGER change_project_insert AFTER INSERT ON project BEGIN
    INSERT OR REPLACE INTO change_log (tbl, k1, k2, origin)
    VALUES ('project', NEW.id, x'', (SELECT value FROM sync_state WHERE key = 'merging_from'));
END;
CREATE TRIGGER change_project_update AFTER UPDATE ON project BEGIN
    INSERT OR REPLACE INTO change_log (tbl, k1, k2, origin)
    VALUES ('project', NEW.id, x'', (SELECT value FROM sync_state WHERE key = 'merging_from'));
END;
CREATE TRIGGER change_project_reference_insert AFTER INSERT ON project_reference BEGIN
    INSERT OR REPLACE INTO change_log (tbl, k1, k2, origin)
    VALUES ('project_reference', NEW.project_id, NEW.reference_id, (SELECT value FROM sync_state WHERE key = 'merging_from'));
END;
CREATE TRIGGER change_project_reference_update AFTER UPDATE ON project_reference BEGIN
    INSERT OR REPLACE INTO change_log (tbl, k1, k2, origin)
    VALUES ('project_reference', NEW.project_id, NEW.reference_id, (SELECT value FROM sync_state WHERE key = 'merging_from'));
END;
CREATE TRIGGER change_tag_insert AFTER INSERT ON tag BEGIN
    INSERT OR REPLACE INTO change_log (tbl, k1, k2, origin)
    VALUES ('tag', NEW.id, x'', (SELECT value FROM sync_state WHERE key = 'merging_from'));
END;
CREATE TRIGGER change_tag_update AFTER UPDATE ON tag BEGIN
    INSERT OR REPLACE INTO change_log (tbl, k1, k2, origin)
    VALUES ('tag', NEW.id, x'', (SELECT value FROM sync_state WHERE key = 'merging_from'));
END;
CREATE TRIGGER change_reference_tag_insert AFTER INSERT ON reference_tag BEGIN
    INSERT OR REPLACE INTO change_log (tbl, k1, k2, origin)
    VALUES ('reference_tag', NEW.reference_id, NEW.tag_id, (SELECT value FROM sync_state WHERE key = 'merging_from'));
END;
CREATE TRIGGER change_reference_tag_update AFTER UPDATE ON reference_tag BEGIN
    INSERT OR REPLACE INTO change_log (tbl, k1, k2, origin)
    VALUES ('reference_tag', NEW.reference_id, NEW.tag_id, (SELECT value FROM sync_state WHERE key = 'merging_from'));
END;
CREATE TRIGGER change_note_insert AFTER INSERT ON note BEGIN
    INSERT OR REPLACE INTO change_log (tbl, k1, k2, origin)
    VALUES ('note', NEW.id, x'', (SELECT value FROM sync_state WHERE key = 'merging_from'));
END;
CREATE TRIGGER change_note_update AFTER UPDATE ON note BEGIN
    INSERT OR REPLACE INTO change_log (tbl, k1, k2, origin)
    VALUES ('note', NEW.id, x'', (SELECT value FROM sync_state WHERE key = 'merging_from'));
END;
CREATE TRIGGER change_highlight_insert AFTER INSERT ON highlight BEGIN
    INSERT OR REPLACE INTO change_log (tbl, k1, k2, origin)
    VALUES ('highlight', NEW.id, x'', (SELECT value FROM sync_state WHERE key = 'merging_from'));
END;
CREATE TRIGGER change_highlight_update AFTER UPDATE ON highlight BEGIN
    INSERT OR REPLACE INTO change_log (tbl, k1, k2, origin)
    VALUES ('highlight', NEW.id, x'', (SELECT value FROM sync_state WHERE key = 'merging_from'));
END;
CREATE TRIGGER change_reference_relation_insert AFTER INSERT ON reference_relation BEGIN
    INSERT OR REPLACE INTO change_log (tbl, k1, k2, origin)
    VALUES ('reference_relation', NEW.a, NEW.b, (SELECT value FROM sync_state WHERE key = 'merging_from'));
END;
CREATE TRIGGER change_reference_relation_update AFTER UPDATE ON reference_relation BEGIN
    INSERT OR REPLACE INTO change_log (tbl, k1, k2, origin)
    VALUES ('reference_relation', NEW.a, NEW.b, (SELECT value FROM sync_state WHERE key = 'merging_from'));
END;
CREATE TRIGGER change_attachment_insert AFTER INSERT ON attachment BEGIN
    INSERT OR REPLACE INTO change_log (tbl, k1, k2, origin)
    VALUES ('attachment', NEW.id, x'', (SELECT value FROM sync_state WHERE key = 'merging_from'));
END;
CREATE TRIGGER change_attachment_update AFTER UPDATE ON attachment BEGIN
    INSERT OR REPLACE INTO change_log (tbl, k1, k2, origin)
    VALUES ('attachment', NEW.id, x'', (SELECT value FROM sync_state WHERE key = 'merging_from'));
END;
CREATE TRIGGER change_duplicate_dismissal_insert AFTER INSERT ON duplicate_dismissal BEGIN
    INSERT OR REPLACE INTO change_log (tbl, k1, k2, origin)
    VALUES ('duplicate_dismissal', NEW.a, NEW.b, (SELECT value FROM sync_state WHERE key = 'merging_from'));
END;
CREATE TRIGGER change_duplicate_dismissal_update AFTER UPDATE ON duplicate_dismissal BEGIN
    INSERT OR REPLACE INTO change_log (tbl, k1, k2, origin)
    VALUES ('duplicate_dismissal', NEW.a, NEW.b, (SELECT value FROM sync_state WHERE key = 'merging_from'));
END;

-- Everything already here counts as changed, so the first sync sends it.
INSERT OR IGNORE INTO change_log (tbl, k1, k2) SELECT 'reference', id, x'' FROM reference;
INSERT OR IGNORE INTO change_log (tbl, k1, k2) SELECT 'project', id, x'' FROM project;
INSERT OR IGNORE INTO change_log (tbl, k1, k2) SELECT 'project_reference', project_id, reference_id FROM project_reference;
INSERT OR IGNORE INTO change_log (tbl, k1, k2) SELECT 'tag', id, x'' FROM tag;
INSERT OR IGNORE INTO change_log (tbl, k1, k2) SELECT 'reference_tag', reference_id, tag_id FROM reference_tag;
INSERT OR IGNORE INTO change_log (tbl, k1, k2) SELECT 'note', id, x'' FROM note;
INSERT OR IGNORE INTO change_log (tbl, k1, k2) SELECT 'highlight', id, x'' FROM highlight;
INSERT OR IGNORE INTO change_log (tbl, k1, k2) SELECT 'reference_relation', a, b FROM reference_relation;
INSERT OR IGNORE INTO change_log (tbl, k1, k2) SELECT 'attachment', id, x'' FROM attachment;
INSERT OR IGNORE INTO change_log (tbl, k1, k2) SELECT 'duplicate_dismissal', a, b FROM duplicate_dismissal;
