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

allow_tables_to_appear_in_same_query!(histories, namespaces, objects, stats,);
