// @generated automatically by Diesel CLI.

diesel::table! {
    entities (id) {
        id -> Int8,
        entity_type -> Int2,
    }
}

diesel::table! {
    objects (id) {
        id -> Int8,
        kind -> Text,
        value -> Text,
    }
}

diesel::table! {
    predicates (id) {
        id -> Int8,
        iri -> Text,
    }
}

diesel::table! {
    prefixes (namespace) {
        namespace -> Text,
        prefix -> Text,
    }
}

diesel::table! {
    properties (subject, predicate, literal_value) {
        subject -> Int8,
        predicate -> Int8,
        literal_value -> Text,
        literal_type -> Nullable<Text>,
    }
}

diesel::table! {
    quoted_properties (id) {
        id -> Int8,
        subject -> Int8,
        predicate -> Int8,
        literal_value -> Text,
        literal_type -> Nullable<Text>,
    }
}

diesel::table! {
    quoted_relations (id) {
        id -> Int8,
        subject -> Int8,
        predicate -> Int8,
        object -> Int8,
    }
}

diesel::table! {
    relations (subject, predicate, object) {
        subject -> Int8,
        predicate -> Int8,
        object -> Int8,
    }
}

diesel::joinable!(objects -> entities (id));
diesel::joinable!(properties -> entities (subject));
diesel::joinable!(properties -> predicates (predicate));
diesel::joinable!(quoted_properties -> predicates (predicate));
diesel::joinable!(quoted_relations -> predicates (predicate));
diesel::joinable!(relations -> predicates (predicate));

diesel::allow_tables_to_appear_in_same_query!(
    entities,
    objects,
    predicates,
    prefixes,
    properties,
    quoted_properties,
    quoted_relations,
    relations,
);
