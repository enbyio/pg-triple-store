CREATE TABLE properties (
  subject       BIGINT NOT NULL REFERENCES objects(id),
  predicate     BIGINT NOT NULL REFERENCES predicates(id),
  literal_value TEXT NOT NULL,
  literal_type  TEXT,
  PRIMARY KEY (subject, predicate, literal_value)
);

CREATE INDEX idx_prop_sp ON properties (subject, predicate);
CREATE INDEX idx_prop_pv ON properties (predicate, literal_value);
CREATE INDEX idx_prop_v ON properties (literal_value);
