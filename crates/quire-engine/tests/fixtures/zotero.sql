-- A small Zotero database for tests: the tables and columns quire reads, as in Zotero's schema
-- 130, with two articles, notes, a collection tree, a PDF and annotations.
CREATE TABLE libraries (libraryID INTEGER PRIMARY KEY, type TEXT NOT NULL);
CREATE TABLE itemTypes (itemTypeID INTEGER PRIMARY KEY, typeName TEXT);
CREATE TABLE fields (fieldID INTEGER PRIMARY KEY, fieldName TEXT);
CREATE TABLE items (itemID INTEGER PRIMARY KEY, itemTypeID INT NOT NULL, libraryID INT NOT NULL, key TEXT NOT NULL);
CREATE TABLE itemDataValues (valueID INTEGER PRIMARY KEY, value UNIQUE);
CREATE TABLE itemData (itemID INT, fieldID INT, valueID, PRIMARY KEY (itemID, fieldID));
CREATE TABLE creators (creatorID INTEGER PRIMARY KEY, firstName TEXT, lastName TEXT, fieldMode INT);
CREATE TABLE creatorTypes (creatorTypeID INTEGER PRIMARY KEY, creatorType TEXT);
CREATE TABLE itemCreators (itemID INT NOT NULL, creatorID INT NOT NULL, creatorTypeID INT NOT NULL DEFAULT 1, orderIndex INT NOT NULL DEFAULT 0);
CREATE TABLE collections (collectionID INTEGER PRIMARY KEY, collectionName TEXT NOT NULL, parentCollectionID INT, libraryID INT NOT NULL, key TEXT NOT NULL);
CREATE TABLE collectionItems (collectionID INT NOT NULL, itemID INT NOT NULL, orderIndex INT NOT NULL DEFAULT 0);
CREATE TABLE tags (tagID INTEGER PRIMARY KEY, name TEXT NOT NULL UNIQUE);
CREATE TABLE itemTags (itemID INT NOT NULL, tagID INT NOT NULL, type INT NOT NULL);
CREATE TABLE itemNotes (itemID INTEGER PRIMARY KEY, parentItemID INT, note TEXT, title TEXT);
CREATE TABLE itemAttachments (itemID INTEGER PRIMARY KEY, parentItemID INT, linkMode INT, contentType TEXT, path TEXT);
CREATE TABLE itemAnnotations (itemID INTEGER PRIMARY KEY, parentItemID INT NOT NULL, type INTEGER NOT NULL, text TEXT, comment TEXT, pageLabel TEXT, sortIndex TEXT NOT NULL);
CREATE TABLE deletedItems (itemID INTEGER PRIMARY KEY);
CREATE TABLE deletedCollections (collectionID INTEGER PRIMARY KEY);
CREATE TABLE itemRelations (itemID INT NOT NULL, predicateID INT NOT NULL, object TEXT NOT NULL);
CREATE TABLE relationPredicates (predicateID INTEGER PRIMARY KEY, predicate TEXT UNIQUE);

INSERT INTO libraries VALUES (1, 'user'), (2, 'group');
INSERT INTO itemTypes VALUES (1, 'journalArticle'), (2, 'note'), (3, 'attachment'), (4, 'annotation'), (5, 'book');
INSERT INTO fields VALUES (1, 'title'), (2, 'DOI'), (3, 'date'), (4, 'publicationTitle'), (5, 'extra');
INSERT INTO creatorTypes VALUES (1, 'author'), (2, 'editor');
INSERT INTO relationPredicates VALUES (1, 'dc:relation');

INSERT INTO items VALUES
    (1, 1, 1, 'AAAA1111'), (2, 1, 1, 'BBBB2222'), (3, 2, 1, 'NOTE3333'), (4, 3, 1, 'PDF44444'),
    (5, 4, 1, 'ANNO5555'), (6, 4, 1, 'ANNO6666'), (7, 5, 1, 'GONE7777'), (8, 1, 2, 'GRUP8888'),
    (9, 2, 1, 'NOTE9999');
INSERT INTO deletedItems VALUES (7);
INSERT INTO itemDataValues VALUES
    (1, 'Nanometre-scale thermometry in a living cell'), (2, '10.1038/nature12373'),
    (3, '2013-07-31 2013-07-31'), (4, 'Nature'), (5, 'A paper without a DOI'), (6, '2020-00-00 2020'),
    (7, 'Citation Key: plainPaper2020'), (8, 'Deleted book'), (9, 'Group paper');
INSERT INTO itemData VALUES (1, 1, 1), (1, 2, 2), (1, 3, 3), (1, 4, 4), (2, 1, 5), (2, 3, 6), (2, 5, 7),
    (7, 1, 8), (8, 1, 9);
INSERT INTO creators VALUES (1, 'G.', 'Kucsko', 0), (2, '', 'Example Consortium', 1);
INSERT INTO itemCreators VALUES (1, 1, 1, 0), (2, 2, 1, 0);
INSERT INTO collections VALUES (1, 'Thesis', NULL, 1, 'COLL0001'), (2, 'Chapter 2', 1, 1, 'COLL0002'),
    (3, 'Old', NULL, 1, 'COLL0003');
INSERT INTO deletedCollections VALUES (3);
INSERT INTO collectionItems VALUES (2, 1, 0), (1, 2, 0), (3, 1, 0), (1, 7, 1);
INSERT INTO tags VALUES (1, 'to-read'), (2, 'Thermometry');
INSERT INTO itemTags VALUES (1, 1, 0), (1, 2, 1);
INSERT INTO itemNotes VALUES (3, 1, '<div><p>Key idea: <b>NV</b> centres</p></div>', 'Key idea'),
    (9, NULL, '<p>A standalone note</p>', 'A standalone note');
INSERT INTO itemAttachments VALUES (4, 1, 0, 'application/pdf', 'storage:kucsko.pdf');
INSERT INTO itemAnnotations VALUES (5, 4, 1, 'quantum spin defects', 'useful', '55', '00001'),
    (6, 4, 3, NULL, NULL, '56', '00002');
INSERT INTO itemRelations VALUES (1, 1, 'http://zotero.org/users/local/abcd/items/BBBB2222');
