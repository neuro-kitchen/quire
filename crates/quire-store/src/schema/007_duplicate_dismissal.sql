-- Pairs of references the user said are not duplicates (`quire ref distinct`), so they are not
-- reported again. Stored once with a < b (by ID bytes), like reference_relation.
CREATE TABLE duplicate_dismissal (
    a           BLOB NOT NULL REFERENCES reference(id),
    b           BLOB NOT NULL REFERENCES reference(id),
    created_at  INTEGER NOT NULL,
    updated_by  BLOB NOT NULL,
    PRIMARY KEY (a, b),
    CHECK (a < b)
) STRICT;
