CREATE TABLE relations (
  subject   BIGINT NOT NULL REFERENCES objects(id),
  predicate BIGINT NOT NULL REFERENCES predicates(id),
  object    BIGINT NOT NULL REFERENCES objects(id),
  PRIMARY KEY (subject, predicate, object)
);

CREATE INDEX idx_spo ON relations (subject, predicate, object);
CREATE INDEX idx_ops ON relations (object, predicate, subject);
CREATE INDEX idp_pos ON relations (predicate, object, subject);
