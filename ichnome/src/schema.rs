table! {
    histories (id) {
        id -> Integer,
        namespace_id -> Varchar,
        path -> Varchar,
        version -> Integer,
        status -> Integer,
        mtime -> Nullable<Datetime>,
        object_id -> Nullable<Integer>,
        digest -> Nullable<Char>,
        created_at -> Datetime,
        updated_at -> Datetime,
    }
}

table! {
    namespaces (id) {
        id -> Varchar,
        url -> Varchar,
        description -> Varchar,
        history_id -> Integer,
        version -> Integer,
        status -> Integer,
        mtime -> Nullable<Datetime>,
        object_id -> Nullable<Integer>,
        digest -> Nullable<Char>,
        size -> Nullable<Bigint>,
        fast_digest -> Nullable<Bigint>,
        created_at -> Datetime,
        updated_at -> Datetime,
    }
}

table! {
    objects (id) {
        id -> Integer,
        digest -> Char,
        size -> Bigint,
        fast_digest -> Bigint,
        git_object_id -> Char,
    }
}

table! {
    stats (id) {
        id -> Integer,
        namespace_id -> Varchar,
        path -> Varchar,
        history_id -> Integer,
        version -> Integer,
        status -> Integer,
        mtime -> Nullable<Datetime>,
        object_id -> Nullable<Integer>,
        digest -> Nullable<Char>,
        size -> Nullable<Bigint>,
        fast_digest -> Nullable<Bigint>,
        created_at -> Datetime,
        updated_at -> Datetime,
    }
}

allow_tables_to_appear_in_same_query!(histories, namespaces, objects, stats,);
