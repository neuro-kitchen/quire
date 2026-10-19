-- Full-text search (FTS5) over what a reference is and what the user wrote about it. Rows are
-- keyed by the reference's rowid and kept current by triggers, so every way of changing a
-- reference, its tags, notes or highlights updates the index; references in the trash are left
-- out. Accents and case are ignored ("Müller" finds "muller").
CREATE VIRTUAL TABLE search_index USING fts5(
    cite_key, title, authors, abstract, venue, identifiers, tags, notes,
    tokenize = 'unicode61 remove_diacritics 2'
);

-- A paper's text (the reader's stored text), written by the engine when it extracts it.
CREATE VIRTUAL TABLE paper_text_index USING fts5(
    body,
    tokenize = 'unicode61 remove_diacritics 2'
);

-- What the index holds for each live reference.
CREATE VIEW search_row AS
SELECT
    r.rowid AS rowid,
    r.cite_key AS cite_key,
    coalesce(r.title, '') AS title,
    coalesce((SELECT group_concat(trim(coalesce(json_extract(c.value, '$.given'), '') || ' '
                  || coalesce(json_extract(c.value, '$.family'), '') || coalesce(json_extract(c.value, '$.literal'), '')), '; ')
              FROM json_each(r.data, '$.creators') c), '') AS authors,
    coalesce(json_extract(r.data, '$.text.abstract'), '') AS abstract,
    coalesce(json_extract(r.data, '$.text."container-title"'), '') AS venue,
    coalesce((SELECT group_concat(json_extract(i.value, '$.value'), ' ') FROM json_each(r.data, '$.identifiers') i), '') AS identifiers,
    coalesce((SELECT group_concat(t.name, ' ') FROM reference_tag rt JOIN tag t ON t.id = rt.tag_id
              WHERE rt.reference_id = r.id AND rt.deleted_at IS NULL AND t.deleted_at IS NULL), '') AS tags,
    coalesce((SELECT group_concat(n.body, ' ') FROM note n WHERE n.reference_id = r.id AND n.deleted_at IS NULL), '')
        || ' ' ||
    coalesce((SELECT group_concat(h.quote || ' ' || coalesce(h.comment, ''), ' ') FROM highlight h
              WHERE h.reference_id = r.id AND h.deleted_at IS NULL), '') AS notes
FROM reference r
WHERE r.deleted_at IS NULL;

INSERT INTO search_index (rowid, cite_key, title, authors, abstract, venue, identifiers, tags, notes)
SELECT rowid, cite_key, title, authors, abstract, venue, identifiers, tags, notes FROM search_row;

-- Re-indexing one reference: delete its row, insert it again from the view (nothing when trashed).
CREATE TRIGGER search_reference_insert AFTER INSERT ON reference BEGIN
    INSERT INTO search_index (rowid, cite_key, title, authors, abstract, venue, identifiers, tags, notes)
    SELECT rowid, cite_key, title, authors, abstract, venue, identifiers, tags, notes FROM search_row WHERE rowid = NEW.rowid;
END;
CREATE TRIGGER search_reference_update AFTER UPDATE ON reference BEGIN
    DELETE FROM search_index WHERE rowid = NEW.rowid;
    INSERT INTO search_index (rowid, cite_key, title, authors, abstract, venue, identifiers, tags, notes)
    SELECT rowid, cite_key, title, authors, abstract, venue, identifiers, tags, notes FROM search_row WHERE rowid = NEW.rowid;
END;

-- Tags, notes and highlights change the row of the reference they belong to.
CREATE TRIGGER search_tag_link AFTER INSERT ON reference_tag BEGIN
    UPDATE reference SET rowid = rowid WHERE id = NEW.reference_id;
END;
CREATE TRIGGER search_tag_link_update AFTER UPDATE ON reference_tag BEGIN
    UPDATE reference SET rowid = rowid WHERE id = NEW.reference_id;
END;
CREATE TRIGGER search_tag_update AFTER UPDATE ON tag BEGIN
    UPDATE reference SET rowid = rowid WHERE id IN (SELECT reference_id FROM reference_tag WHERE tag_id = NEW.id);
END;
CREATE TRIGGER search_note_insert AFTER INSERT ON note BEGIN
    UPDATE reference SET rowid = rowid WHERE id = NEW.reference_id;
END;
CREATE TRIGGER search_note_update AFTER UPDATE ON note BEGIN
    UPDATE reference SET rowid = rowid WHERE id IN (NEW.reference_id, OLD.reference_id);
END;
CREATE TRIGGER search_highlight_insert AFTER INSERT ON highlight BEGIN
    UPDATE reference SET rowid = rowid WHERE id = NEW.reference_id;
END;
CREATE TRIGGER search_highlight_update AFTER UPDATE ON highlight BEGIN
    UPDATE reference SET rowid = rowid WHERE id IN (NEW.reference_id, OLD.reference_id);
END;
