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

#[derive(Debug, Insertable)]
#[table_name = "namespaces"]
pub struct NamespaceInsertForm<'a> {
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

    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, AsChangeset)]
#[table_name = "namespaces"]
#[changeset_options(treat_none_as_null = "true")]
pub struct NamespaceUpdateForm<'a> {
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

    pub updated_at: NaiveDateTime,
}

impl<'a> From<&'a Namespace> for NamespaceUpdateForm<'a> {
    fn from(src: &'a Namespace) -> Self {
        NamespaceUpdateForm {
            url: &src.url,
            type_: src.type_,
            history_id: src.hisotry_id,
            version: src.version,
            status: src.status,
            mtime: src.mtime,
            object_id: src.object_id,
            digest: src.digest.as_ref().map(|s| s.as_str()),
            size: src.size,
            fast_digest: src.fast_digest,
            updated_at: src.updated_at,
        }
    }
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

#[derive(Debug, Insertable)]
#[table_name = "stats"]
pub struct StatInsertForm<'a> {
    pub namespace_id: &'a str,
    pub path: &'a str,

    pub history_id: i32,

    pub version: i32,
    pub status: i32,
    pub mtime: Option<NaiveDateTime>,
    pub object_id: Option<i32>,

    pub digest: Option<&'a str>,
    pub size: Option<i64>,
    pub fast_digest: Option<i64>,

    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, AsChangeset)]
#[table_name = "stats"]
#[changeset_options(treat_none_as_null = "true")]
pub struct StatUpdateForm<'a> {
    pub history_id: i32,

    pub version: i32,
    pub status: i32,
    pub mtime: Option<NaiveDateTime>,
    pub object_id: Option<i32>,

    pub digest: Option<&'a str>,
    pub size: Option<i64>,
    pub fast_digest: Option<i64>,

    pub updated_at: NaiveDateTime,
}

impl<'a> From<&'a Stat> for StatUpdateForm<'a> {
    fn from(src: &'a Stat) -> Self {
        StatUpdateForm {
            history_id: src.history_id,
            version: src.version,
            status: src.status,
            mtime: src.mtime,
            object_id: src.object_id,
            digest: src.digest.as_ref().map(|s| s.as_str()),
            size: src.size,
            fast_digest: src.fast_digest,
            updated_at: src.updated_at,
        }
    }
}
