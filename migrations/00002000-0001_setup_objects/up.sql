CREATE TABLE objects (
  id    BIGINT PRIMARY KEY REFERENCES entities(id) ON DELETE CASCADE,
  kind  TEXT NOT NULL CHECK (kind IN ('iri', 'blank')),
  value TEXT NOT NULL,
  UNIQUE (kind, value)
);
