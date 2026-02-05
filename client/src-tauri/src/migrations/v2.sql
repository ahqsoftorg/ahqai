DROP TABLE IF EXISTS MESSAGES_SEARCH;
DROP TRIGGER IF EXISTS messages_ai;
DROP TRIGGER IF EXISTS messages_ad;
DROP TRIGGER IF EXISTS messages_au;

CREATE VIRTUAL TABLE MESSAGES_SEARCH USING fts5(
  content,
  content = 'MESSAGES',
  content_rowid = 'id',
  tokenize = 'trigram'
);

INSERT INTO MESSAGES_SEARCH(MESSAGES_SEARCH) VALUES('rebuild');

-- Sync Insert
CREATE TRIGGER messages_ai AFTER INSERT ON MESSAGES 
BEGIN
  INSERT INTO MESSAGES_SEARCH(rowid, content) VALUES (new.id, new.content);
END;

-- Sync Delete
CREATE TRIGGER messages_ad BEFORE DELETE ON MESSAGES 
BEGIN
  INSERT INTO MESSAGES_SEARCH(MESSAGES_SEARCH, rowid, content) VALUES('delete', old.id, old.content);
END;

-- Sync Update
CREATE TRIGGER messages_au AFTER UPDATE ON MESSAGES BEGIN
  INSERT INTO MESSAGES_SEARCH(MESSAGES_SEARCH, rowid, content) VALUES('delete', old.id, old.content);
  INSERT INTO MESSAGES_SEARCH(rowid, content) VALUES (new.id, new.content);
END;