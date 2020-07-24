use std::error::Error;

use diesel::prelude::*;

use crate::models::{
    History, HistoryInsertForm, Namespace, NamespaceInsertForm, NamespaceUpdateForm, Object, ObjectInsertForm, Stat,
    StatInsertForm, StatUpdateForm,
};

embed_migrations!("migrations");

pub fn migrate(conn: &SqliteConnection) -> Result<(), Box<dyn Error>> {
    embedded_migrations::run(conn)?;
    Ok(())
}

pub struct MysqlObjects;

impl MysqlObjects {
    pub fn find(conn: &MysqlConnection, id: i32) -> Result<Option<Object>, Box<dyn Error>> {
        use crate::db::schema::objects::dsl;
        let q = dsl::objects.find(id);
        Ok(q.first::<Object>(conn).optional()?)
    }

    pub fn find_by_digest(conn: &MysqlConnection, digest: &str) -> Result<Option<Object>, Box<dyn Error>> {
        use crate::db::schema::objects::dsl;
        let q = dsl::objects.filter(dsl::digest.eq(digest));
        Ok(q.first::<Object>(conn).optional()?)
    }

    pub fn insert(conn: &MysqlConnection, object_form: &ObjectInsertForm) -> Result<(), Box<dyn Error>> {
        use crate::db::schema::objects::dsl;
        let q = diesel::insert_into(dsl::objects).values(object_form);
        q.execute(conn)?;
        Ok(())
    }

    pub fn insert_and_find(conn: &MysqlConnection, object_form: &ObjectInsertForm) -> Result<Object, Box<dyn Error>> {
        Self::insert(conn, object_form)?;
        let object = Self::find_by_digest(conn, object_form.digest)?;
        Ok(object.unwrap())
    }
}

pub struct MysqlHistories;

impl MysqlHistories {
    pub fn find(conn: &MysqlConnection, id: i32) -> Result<Option<History>, Box<dyn Error>> {
        use crate::db::schema::histories::dsl;
        let q = dsl::histories.find(id);
        Ok(q.first::<History>(conn).optional()?)
    }

    pub fn find_latest_by_path(
        conn: &MysqlConnection,
        namespace_id: &str,
        path: &str,
    ) -> Result<Option<History>, Box<dyn Error>> {
        use crate::db::schema::histories::dsl;
        let q = dsl::histories
            .filter(dsl::namespace_id.eq(namespace_id))
            .filter(dsl::path.eq(path))
            .order(dsl::version.desc())
            .limit(1);
        Ok(q.first::<History>(conn).optional()?)
    }

    pub fn find_by_path_and_version(
        conn: &MysqlConnection,
        namespace_id: &str,
        path: &str,
        version: i32,
    ) -> Result<Option<History>, Box<dyn Error>> {
        use crate::db::schema::histories::dsl;
        let q = dsl::histories
            .filter(dsl::namespace_id.eq(namespace_id))
            .filter(dsl::path.eq(path))
            .filter(dsl::version.eq(version));
        Ok(q.first::<History>(conn).optional()?)
    }

    pub fn select_by_path(
        conn: &MysqlConnection,
        namespace_id: &str,
        path: &str,
    ) -> Result<Vec<History>, Box<dyn Error>> {
        use crate::db::schema::histories::dsl;
        let q = dsl::histories
            .filter(dsl::namespace_id.eq(namespace_id))
            .filter(dsl::path.eq(path))
            .order(dsl::version.asc());
        let histories = q.load::<History>(conn)?;
        Ok(histories)
    }

    pub fn insert(conn: &MysqlConnection, history_form: &HistoryInsertForm) -> Result<(), Box<dyn Error>> {
        use crate::db::schema::histories::dsl;
        let q = diesel::insert_into(dsl::histories).values(history_form);
        q.execute(conn)?;
        Ok(())
    }

    pub fn insert_and_find(
        conn: &MysqlConnection,
        history_form: &HistoryInsertForm,
    ) -> Result<History, Box<dyn Error>> {
        Self::insert(conn, history_form)?;
        let history =
            Self::find_by_path_and_version(conn, history_form.namespace_id, history_form.path, history_form.version)?;
        Ok(history.unwrap())
    }
}

pub struct MysqlNamespaces;

impl MysqlNamespaces {
    pub fn find(conn: &MysqlConnection, id: &str) -> Result<Option<Namespace>, Box<dyn Error>> {
        use crate::db::schema::namespaces::dsl;
        let q = dsl::namespaces.find(id);
        Ok(q.first::<Namespace>(conn).optional()?)
    }

    pub fn insert(conn: &MysqlConnection, namespace_form: &NamespaceInsertForm) -> Result<(), Box<dyn Error>> {
        use crate::db::schema::namespaces::dsl;
        let q = diesel::insert_into(dsl::namespaces).values(namespace_form);
        q.execute(conn)?;
        Ok(())
    }

    pub fn insert_and_find(
        conn: &MysqlConnection,
        namespace_form: &NamespaceInsertForm,
    ) -> Result<Namespace, Box<dyn Error>> {
        Self::insert(conn, namespace_form)?;
        let namespace = Self::find(conn, namespace_form.id)?;
        Ok(namespace.unwrap())
    }

    pub fn update(
        conn: &MysqlConnection,
        id: &str,
        namespace_form: &NamespaceUpdateForm,
    ) -> Result<(), Box<dyn Error>> {
        use crate::db::schema::namespaces::dsl;
        let q = diesel::update(dsl::namespaces.find(id)).set(namespace_form);
        let n = q.execute(conn)?;
        assert_eq!(1, n);
        Ok(())
    }

    pub fn update_and_find(
        conn: &MysqlConnection,
        id: &str,
        namespace_form: &NamespaceUpdateForm,
    ) -> Result<Namespace, Box<dyn Error>> {
        Self::update(conn, id, namespace_form)?;
        let namespace = Self::find(conn, id)?;
        Ok(namespace.unwrap())
    }
}

pub struct MysqlStats;

impl MysqlStats {
    pub fn find(conn: &MysqlConnection, id: i32) -> Result<Option<Stat>, Box<dyn Error>> {
        use crate::db::schema::stats::dsl;
        let q = dsl::stats.find(id);
        Ok(q.first::<Stat>(conn).optional()?)
    }

    pub fn find_by_path(
        conn: &MysqlConnection,
        namespace_id: &str,
        path: &str,
    ) -> Result<Option<Stat>, Box<dyn Error>> {
        use crate::db::schema::stats::dsl;
        let q = dsl::stats.filter(dsl::namespace_id.eq(namespace_id)).filter(dsl::path.eq(path));
        Ok(q.first::<Stat>(conn).optional()?)
    }

    pub fn select(conn: &MysqlConnection, namespace_id: &str) -> Result<Vec<Stat>, Box<dyn Error>> {
        use crate::db::schema::stats::dsl;
        let q =
            dsl::stats.filter(dsl::namespace_id.eq(namespace_id)).order(dsl::namespace_id.asc()).order(dsl::path.asc());
        let stats = q.load::<Stat>(conn)?;
        Ok(stats)
    }

    pub fn insert(conn: &MysqlConnection, stat_form: &StatInsertForm) -> Result<(), Box<dyn Error>> {
        use crate::db::schema::stats::dsl;
        let q = diesel::insert_into(dsl::stats).values(stat_form);
        q.execute(conn)?;
        Ok(())
    }

    pub fn insert_and_find(conn: &MysqlConnection, stat_form: &StatInsertForm) -> Result<Stat, Box<dyn Error>> {
        Self::insert(conn, stat_form)?;
        let stat = Self::find_by_path(conn, stat_form.namespace_id, stat_form.path)?;
        Ok(stat.unwrap())
    }

    pub fn update(conn: &MysqlConnection, id: i32, stat_form: &StatUpdateForm) -> Result<(), Box<dyn Error>> {
        use crate::db::schema::stats::dsl;
        let q = diesel::update(dsl::stats.find(id)).set(stat_form);
        let n = q.execute(conn)?;
        assert_eq!(1, n);
        Ok(())
    }

    pub fn update_and_find(
        conn: &MysqlConnection,
        id: i32,
        stat_form: &StatUpdateForm,
    ) -> Result<Stat, Box<dyn Error>> {
        Self::update(conn, id, stat_form)?;
        let stat = Self::find(conn, id)?;
        Ok(stat.unwrap())
    }
}
