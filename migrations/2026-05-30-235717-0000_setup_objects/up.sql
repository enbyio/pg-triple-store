CREATE TABLE objects (
  id    BIGSERIAL PRIMARY KEY,
  kind  TEXT NOT NULL CHECK (kind IN ('iri', 'blank')),
  value TEXT NOT NULL,
  UNIQUE (kind, value)
);
