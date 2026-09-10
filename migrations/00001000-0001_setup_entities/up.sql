CREATE TABLE entities (
    id          BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    entity_type SMALLINT NOT NULL CHECK (entity_type in (1, 2, 3))
);
-- entity_type: 1 -> object; 2 -> quoted_relation; 3 -> quoted_property
