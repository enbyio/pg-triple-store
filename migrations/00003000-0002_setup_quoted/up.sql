CREATE TABLE quoted_relations (
    id          BIGINT PRIMARY KEY REFERENCES entities(id) ON DELETE CASCADE,
    subject     BIGINT NOT NULL REFERENCES entities(id),
    predicate   BIGINT NOT NULL REFERENCES predicates(id),
    object      BIGINT NOT NULL REFERENCES entities(id),
    UNIQUE (subject, predicate, object)
);

CREATE TABLE quoted_properties (
    id              BIGINT PRIMARY KEY REFERENCES entities(id) ON DELETE CASCADE,
    subject         BIGINT NOT NULL REFERENCES entities(id),
    predicate       BIGINT NOT NULL REFERENCES predicates(id),
    literal_value   TEXT NOT NULL,
    literal_type    TEXT,
    UNIQUE(subject, predicate, literal_value)
);
