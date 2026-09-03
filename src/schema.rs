// @generated automatically by Diesel CLI.

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
    relations (subject, predicate, object) {
        subject -> Int8,
        predicate -> Int8,
        object -> Int8,
    }
}

diesel::joinable!(properties -> objects (subject));
diesel::joinable!(properties -> predicates (predicate));
diesel::joinable!(relations -> predicates (predicate));

diesel::allow_tables_to_appear_in_same_query!(objects, predicates, prefixes, properties, relations,);
