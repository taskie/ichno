use chrono::NaiveDateTime;

use crate::db::schema::{histories, namespaces, objects, stats};

#[derive(Clone, Debug, PartialEq, Identifiable, Queryable)]
#[table_name = "objects"]
pub struct Object {
    pub id: i32,

    pub digest: String,
    pub size: i64,
    pub fast_digest: i64,
    pub git_object_id: String,
}

#[derive(Debug, Insertable)]
#[table_name = "objects"]
pub struct ObjectInsertForm<'a> {
    pub digest: &'a str,
    pub size: i64,
    pub fast_digest: i64,
    pub git_object_id: &'a str,
}

#[derive(Clone, Debug, PartialEq, Identifiable, Queryable)]
#[table_name = "histories"]
pub struct History {
    pub id: i32,

    pub namespace_id: String,
    pub path: String,
    pub version: i32,

    pub status: i32,
    pub mtime: Option<NaiveDateTime>,
    pub object_id: Option<i32>,
    pub digest: Option<String>,

    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Insertable)]
#[table_name = "histories"]
pub struct HistoryInsertForm<'a> {
    pub namespace_id: &'a str,
    pub path: &'a str,
    pub version: i32,

    pub status: i32,
    pub mtime: Option<NaiveDateTime>,
    pub object_id: Option<i32>,
    pub digest: Option<&'a str>,

    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Clone, Debug, PartialEq, Identifiable, Queryable)]
#[table_name = "namespaces"]
pub struct Namespace {
    pub id: String,

    pub url: String,
    pub type_: i32,

    pub hisotry_id: Option<i32>,

    pub version: Option<i32>,
    pub status: Option<i32>,
    pub mtime: Option<NaiveDateTime>,
    pub object_id: Option<i32>,

    pub digest: Option<String>,
    pub size: Option<i64>,
    pub fast_digest: Option<i64>,

    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Insertable, Optional)]
#[table_name = "namespaces"]
#[optional(name = "NamespaceUpdateForm", derive = "Default, Debug, AsChangeset")]
pub struct NamespaceInsertForm<'a> {
    #[optional(skip = true)]
    pub id: &'a str,

    pub url: &'a str,
    pub type_: i32,

    pub history_id: Option<i32>,

    pub version: Option<i32>,
    pub status: Option<i32>,
    pub mtime: Option<NaiveDateTime>,
    pub object_id: Option<i32>,

    pub digest: Option<&'a str>,
    pub size: Option<i64>,
    pub fast_digest: Option<i64>,

    #[optional(skip = true)]
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Clone, Debug, PartialEq, Identifiable, Queryable)]
#[table_name = "stats"]
pub struct Stat {
    pub id: i32,

    pub namespace_id: String,
    pub path: String,

    pub history_id: i32,

    pub version: i32,
    pub status: i32,
    pub mtime: Option<NaiveDateTime>,
    pub object_id: Option<i32>,

    pub digest: Option<String>,
    pub size: Option<i64>,
    pub fast_digest: Option<i64>,

    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Insertable, Optional)]
#[table_name = "stats"]
#[optional(name = "StatUpdateForm", derive = "Default, Debug, AsChangeset")]
pub struct StatInsertForm<'a> {
    #[optional(skip = true)]
    pub namespace_id: &'a str,
    #[optional(skip = true)]
    pub path: &'a str,

    pub history_id: i32,

    pub version: i32,
    pub status: i32,
    pub mtime: Option<NaiveDateTime>,
    pub object_id: Option<i32>,

    pub digest: Option<&'a str>,
    pub size: Option<i64>,
    pub fast_digest: Option<i64>,

    #[optional(skip = true)]
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}
