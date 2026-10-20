-- The change-log triggers again, without `INSERT OR REPLACE`: SQLite applies the conflict rule
-- of the statement that fired a trigger to the trigger's own statements, so an upsert
-- (`INSERT … ON CONFLICT DO UPDATE`) on a synced table made `OR REPLACE` fail with a UNIQUE
-- error on `change_log` (found 2026-10-20 setting a reading status twice). Delete, then insert:
-- the row moves to a new `seq`, as before, and no conflict can arise.

DROP TRIGGER change_reference_insert;
CREATE TRIGGER change_reference_insert AFTER INSERT ON reference BEGIN
    DELETE FROM change_log WHERE tbl = 'reference' AND k1 = NEW.id AND k2 = x'';
    INSERT INTO change_log (tbl, k1, k2, origin)
    VALUES ('reference', NEW.id, x'', (SELECT value FROM sync_state WHERE key = 'merging_from'));
END;
DROP TRIGGER change_reference_update;
CREATE TRIGGER change_reference_update AFTER UPDATE ON reference BEGIN
    DELETE FROM change_log WHERE tbl = 'reference' AND k1 = NEW.id AND k2 = x'';
    INSERT INTO change_log (tbl, k1, k2, origin)
    VALUES ('reference', NEW.id, x'', (SELECT value FROM sync_state WHERE key = 'merging_from'));
END;
DROP TRIGGER change_project_insert;
CREATE TRIGGER change_project_insert AFTER INSERT ON project BEGIN
    DELETE FROM change_log WHERE tbl = 'project' AND k1 = NEW.id AND k2 = x'';
    INSERT INTO change_log (tbl, k1, k2, origin)
    VALUES ('project', NEW.id, x'', (SELECT value FROM sync_state WHERE key = 'merging_from'));
END;
DROP TRIGGER change_project_update;
CREATE TRIGGER change_project_update AFTER UPDATE ON project BEGIN
    DELETE FROM change_log WHERE tbl = 'project' AND k1 = NEW.id AND k2 = x'';
    INSERT INTO change_log (tbl, k1, k2, origin)
    VALUES ('project', NEW.id, x'', (SELECT value FROM sync_state WHERE key = 'merging_from'));
END;
DROP TRIGGER change_project_reference_insert;
CREATE TRIGGER change_project_reference_insert AFTER INSERT ON project_reference BEGIN
    DELETE FROM change_log WHERE tbl = 'project_reference' AND k1 = NEW.project_id AND k2 = NEW.reference_id;
    INSERT INTO change_log (tbl, k1, k2, origin)
    VALUES ('project_reference', NEW.project_id, NEW.reference_id, (SELECT value FROM sync_state WHERE key = 'merging_from'));
END;
DROP TRIGGER change_project_reference_update;
CREATE TRIGGER change_project_reference_update AFTER UPDATE ON project_reference BEGIN
    DELETE FROM change_log WHERE tbl = 'project_reference' AND k1 = NEW.project_id AND k2 = NEW.reference_id;
    INSERT INTO change_log (tbl, k1, k2, origin)
    VALUES ('project_reference', NEW.project_id, NEW.reference_id, (SELECT value FROM sync_state WHERE key = 'merging_from'));
END;
DROP TRIGGER change_tag_insert;
CREATE TRIGGER change_tag_insert AFTER INSERT ON tag BEGIN
    DELETE FROM change_log WHERE tbl = 'tag' AND k1 = NEW.id AND k2 = x'';
    INSERT INTO change_log (tbl, k1, k2, origin)
    VALUES ('tag', NEW.id, x'', (SELECT value FROM sync_state WHERE key = 'merging_from'));
END;
DROP TRIGGER change_tag_update;
CREATE TRIGGER change_tag_update AFTER UPDATE ON tag BEGIN
    DELETE FROM change_log WHERE tbl = 'tag' AND k1 = NEW.id AND k2 = x'';
    INSERT INTO change_log (tbl, k1, k2, origin)
    VALUES ('tag', NEW.id, x'', (SELECT value FROM sync_state WHERE key = 'merging_from'));
END;
DROP TRIGGER change_reference_tag_insert;
CREATE TRIGGER change_reference_tag_insert AFTER INSERT ON reference_tag BEGIN
    DELETE FROM change_log WHERE tbl = 'reference_tag' AND k1 = NEW.reference_id AND k2 = NEW.tag_id;
    INSERT INTO change_log (tbl, k1, k2, origin)
    VALUES ('reference_tag', NEW.reference_id, NEW.tag_id, (SELECT value FROM sync_state WHERE key = 'merging_from'));
END;
DROP TRIGGER change_reference_tag_update;
CREATE TRIGGER change_reference_tag_update AFTER UPDATE ON reference_tag BEGIN
    DELETE FROM change_log WHERE tbl = 'reference_tag' AND k1 = NEW.reference_id AND k2 = NEW.tag_id;
    INSERT INTO change_log (tbl, k1, k2, origin)
    VALUES ('reference_tag', NEW.reference_id, NEW.tag_id, (SELECT value FROM sync_state WHERE key = 'merging_from'));
END;
DROP TRIGGER change_note_insert;
CREATE TRIGGER change_note_insert AFTER INSERT ON note BEGIN
    DELETE FROM change_log WHERE tbl = 'note' AND k1 = NEW.id AND k2 = x'';
    INSERT INTO change_log (tbl, k1, k2, origin)
    VALUES ('note', NEW.id, x'', (SELECT value FROM sync_state WHERE key = 'merging_from'));
END;
DROP TRIGGER change_note_update;
CREATE TRIGGER change_note_update AFTER UPDATE ON note BEGIN
    DELETE FROM change_log WHERE tbl = 'note' AND k1 = NEW.id AND k2 = x'';
    INSERT INTO change_log (tbl, k1, k2, origin)
    VALUES ('note', NEW.id, x'', (SELECT value FROM sync_state WHERE key = 'merging_from'));
END;
DROP TRIGGER change_highlight_insert;
CREATE TRIGGER change_highlight_insert AFTER INSERT ON highlight BEGIN
    DELETE FROM change_log WHERE tbl = 'highlight' AND k1 = NEW.id AND k2 = x'';
    INSERT INTO change_log (tbl, k1, k2, origin)
    VALUES ('highlight', NEW.id, x'', (SELECT value FROM sync_state WHERE key = 'merging_from'));
END;
DROP TRIGGER change_highlight_update;
CREATE TRIGGER change_highlight_update AFTER UPDATE ON highlight BEGIN
    DELETE FROM change_log WHERE tbl = 'highlight' AND k1 = NEW.id AND k2 = x'';
    INSERT INTO change_log (tbl, k1, k2, origin)
    VALUES ('highlight', NEW.id, x'', (SELECT value FROM sync_state WHERE key = 'merging_from'));
END;
DROP TRIGGER change_reference_relation_insert;
CREATE TRIGGER change_reference_relation_insert AFTER INSERT ON reference_relation BEGIN
    DELETE FROM change_log WHERE tbl = 'reference_relation' AND k1 = NEW.a AND k2 = NEW.b;
    INSERT INTO change_log (tbl, k1, k2, origin)
    VALUES ('reference_relation', NEW.a, NEW.b, (SELECT value FROM sync_state WHERE key = 'merging_from'));
END;
DROP TRIGGER change_reference_relation_update;
CREATE TRIGGER change_reference_relation_update AFTER UPDATE ON reference_relation BEGIN
    DELETE FROM change_log WHERE tbl = 'reference_relation' AND k1 = NEW.a AND k2 = NEW.b;
    INSERT INTO change_log (tbl, k1, k2, origin)
    VALUES ('reference_relation', NEW.a, NEW.b, (SELECT value FROM sync_state WHERE key = 'merging_from'));
END;
DROP TRIGGER change_attachment_insert;
CREATE TRIGGER change_attachment_insert AFTER INSERT ON attachment BEGIN
    DELETE FROM change_log WHERE tbl = 'attachment' AND k1 = NEW.id AND k2 = x'';
    INSERT INTO change_log (tbl, k1, k2, origin)
    VALUES ('attachment', NEW.id, x'', (SELECT value FROM sync_state WHERE key = 'merging_from'));
END;
DROP TRIGGER change_attachment_update;
CREATE TRIGGER change_attachment_update AFTER UPDATE ON attachment BEGIN
    DELETE FROM change_log WHERE tbl = 'attachment' AND k1 = NEW.id AND k2 = x'';
    INSERT INTO change_log (tbl, k1, k2, origin)
    VALUES ('attachment', NEW.id, x'', (SELECT value FROM sync_state WHERE key = 'merging_from'));
END;
DROP TRIGGER change_duplicate_dismissal_insert;
CREATE TRIGGER change_duplicate_dismissal_insert AFTER INSERT ON duplicate_dismissal BEGIN
    DELETE FROM change_log WHERE tbl = 'duplicate_dismissal' AND k1 = NEW.a AND k2 = NEW.b;
    INSERT INTO change_log (tbl, k1, k2, origin)
    VALUES ('duplicate_dismissal', NEW.a, NEW.b, (SELECT value FROM sync_state WHERE key = 'merging_from'));
END;
DROP TRIGGER change_duplicate_dismissal_update;
CREATE TRIGGER change_duplicate_dismissal_update AFTER UPDATE ON duplicate_dismissal BEGIN
    DELETE FROM change_log WHERE tbl = 'duplicate_dismissal' AND k1 = NEW.a AND k2 = NEW.b;
    INSERT INTO change_log (tbl, k1, k2, origin)
    VALUES ('duplicate_dismissal', NEW.a, NEW.b, (SELECT value FROM sync_state WHERE key = 'merging_from'));
END;
DROP TRIGGER change_reading_insert;
CREATE TRIGGER change_reading_insert AFTER INSERT ON reading BEGIN
    DELETE FROM change_log WHERE tbl = 'reading' AND k1 = NEW.reference_id AND k2 = x'';
    INSERT INTO change_log (tbl, k1, k2, origin)
    VALUES ('reading', NEW.reference_id, x'', (SELECT value FROM sync_state WHERE key = 'merging_from'));
END;
DROP TRIGGER change_reading_update;
CREATE TRIGGER change_reading_update AFTER UPDATE ON reading BEGIN
    DELETE FROM change_log WHERE tbl = 'reading' AND k1 = NEW.reference_id AND k2 = x'';
    INSERT INTO change_log (tbl, k1, k2, origin)
    VALUES ('reading', NEW.reference_id, x'', (SELECT value FROM sync_state WHERE key = 'merging_from'));
END;
DROP TRIGGER change_saved_search_insert;
CREATE TRIGGER change_saved_search_insert AFTER INSERT ON saved_search BEGIN
    DELETE FROM change_log WHERE tbl = 'saved_search' AND k1 = NEW.id AND k2 = x'';
    INSERT INTO change_log (tbl, k1, k2, origin)
    VALUES ('saved_search', NEW.id, x'', (SELECT value FROM sync_state WHERE key = 'merging_from'));
END;
DROP TRIGGER change_saved_search_update;
CREATE TRIGGER change_saved_search_update AFTER UPDATE ON saved_search BEGIN
    DELETE FROM change_log WHERE tbl = 'saved_search' AND k1 = NEW.id AND k2 = x'';
    INSERT INTO change_log (tbl, k1, k2, origin)
    VALUES ('saved_search', NEW.id, x'', (SELECT value FROM sync_state WHERE key = 'merging_from'));
END;
