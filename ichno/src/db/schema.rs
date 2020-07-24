table! {
    attributes (id) {
        id -> Integer,
        entity_type -> Integer,
        entity_id -> Integer,
        namespace_id -> Nullable<Text>,
        path -> Nullable<Text>,
        version -> Nullable<Integer>,
        digest -> Nullable<Text>,
        key -> Text,
        value_object_id -> Integer,
        value_digest -> Text,
        value_content_type -> Integer,
        status -> Integer,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

table! {
    contents (object_id) {
        object_id -> Integer,
        digest -> Text,
        body -> Binary,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

table! {
    histories (id) {
        id -> Integer,
        namespace_id -> Text,
        path -> Text,
        version -> Integer,
        status -> Integer,
        mtime -> Nullable<Timestamp>,
        object_id -> Nullable<Integer>,
        digest -> Nullable<Text>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

table! {
    namespaces (id) {
        id -> Text,
        url -> Text,
        #[sql_name = "type"]
        type_ -> Integer,
        history_id -> Nullable<Integer>,
        version -> Nullable<Integer>,
        status -> Nullable<Integer>,
        mtime -> Nullable<Timestamp>,
        object_id -> Nullable<Integer>,
        digest -> Nullable<Text>,
        size -> Nullable<BigInt>,
        fast_digest -> Nullable<BigInt>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

table! {
    objects (id) {
        id -> Integer,
        digest -> Text,
        size -> BigInt,
        fast_digest -> BigInt,
        git_object_id -> Text,
    }
}

table! {
    stats (id) {
        id -> Integer,
        namespace_id -> Text,
        path -> Text,
        history_id -> Integer,
        version -> Integer,
        status -> Integer,
        mtime -> Nullable<Timestamp>,
        object_id -> Nullable<Integer>,
        digest -> Nullable<Text>,
        size -> Nullable<BigInt>,
        fast_digest -> Nullable<BigInt>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

allow_tables_to_appear_in_same_query!(
    attributes,
    contents,
    histories,
    namespaces,
    objects,
    stats,
);
