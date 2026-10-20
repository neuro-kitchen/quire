-- Sharing projects with other people (`.knowledge/sync.md`): a project's owner (who shared it;
-- set once, the root of trust for its members on every device) and its members' roles.
ALTER TABLE project ADD COLUMN owner TEXT;

CREATE TABLE project_member (
    project_id  BLOB NOT NULL REFERENCES project(id),
    user_id     TEXT NOT NULL,
    name        TEXT NOT NULL,
    role        TEXT NOT NULL,
    granted_by  TEXT NOT NULL,
    created_at  INTEGER NOT NULL,
    updated_at  INTEGER NOT NULL,
    updated_by  BLOB NOT NULL,
    deleted_at  INTEGER,
    PRIMARY KEY (project_id, user_id)
) STRICT;

CREATE TRIGGER change_project_member_insert AFTER INSERT ON project_member BEGIN
    DELETE FROM change_log WHERE tbl = 'project_member' AND k1 = NEW.project_id AND k2 = CAST(NEW.user_id AS BLOB);
    INSERT INTO change_log (tbl, k1, k2, origin)
    VALUES ('project_member', NEW.project_id, CAST(NEW.user_id AS BLOB), (SELECT value FROM sync_state WHERE key = 'merging_from'));
END;
CREATE TRIGGER change_project_member_update AFTER UPDATE ON project_member BEGIN
    DELETE FROM change_log WHERE tbl = 'project_member' AND k1 = NEW.project_id AND k2 = CAST(NEW.user_id AS BLOB);
    INSERT INTO change_log (tbl, k1, k2, origin)
    VALUES ('project_member', NEW.project_id, CAST(NEW.user_id AS BLOB), (SELECT value FROM sync_state WHERE key = 'merging_from'));
END;
