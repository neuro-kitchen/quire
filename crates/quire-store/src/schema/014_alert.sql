-- Alerts: a search of arXiv, PubMed or Crossref checked for new papers, and the works each has
-- shown already (by identifier key), so a check lists only what is new. Kept on this device.
CREATE TABLE alert (
    id          BLOB PRIMARY KEY,
    name        TEXT NOT NULL,
    key         TEXT NOT NULL,
    source      TEXT NOT NULL,
    query       TEXT NOT NULL,
    checked_at  INTEGER,
    created_at  INTEGER NOT NULL,
    updated_at  INTEGER NOT NULL,
    updated_by  BLOB NOT NULL,
    deleted_at  INTEGER
) STRICT;
CREATE UNIQUE INDEX alert_key ON alert(key) WHERE deleted_at IS NULL;

CREATE TABLE alert_seen (
    alert_id  BLOB NOT NULL REFERENCES alert(id),
    work      TEXT NOT NULL,
    seen_at   INTEGER NOT NULL,
    PRIMARY KEY (alert_id, work)
) STRICT;
