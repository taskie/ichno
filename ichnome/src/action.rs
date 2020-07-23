use std::{error::Error, path::Path};

use chrono::{DateTime, TimeZone};
use diesel::{Connection, MysqlConnection, SqliteConnection};
use ichno::{
    consts::Status,
    sqlite::{SqliteHistories, SqliteObjects, SqliteStats},
};
use sha1::Sha1;
use url::Url;

use crate::{
    consts::META_NAMESPACE_ID,
    fs,
    fs::FileMetadata,
    models::{
        HistoryInsertForm, Namespace, NamespaceInsertForm, NamespaceUpdateForm, ObjectInsertForm, Stat, StatInsertForm,
        StatUpdateForm,
    },
    mysql::{MysqlHistories, MysqlNamespaces, MysqlObjects, MysqlStats},
    ssh,
};
use sha1::digest::FixedOutput;

pub struct Context<'c> {
    pub connection: &'c MysqlConnection,
}

#[derive(Debug)]
pub struct RegisterRequest<Tz: TimeZone> {
    pub namespace_id: String,
    pub url: String,
    pub current_time: DateTime<Tz>,
    pub options: RegisterOptions,
}

#[derive(Debug)]
pub struct RegisterOptions {
    pub description: Option<String>,
    pub force: bool,
}

#[derive(Debug)]
pub struct RegisterResponse {
    pub namespace: Namespace,
}

impl Default for RegisterOptions {
    fn default() -> Self {
        RegisterOptions { description: None, force: false }
    }
}

pub fn register<Tz: TimeZone>(ctx: &Context, req: &RegisterRequest<Tz>) -> Result<RegisterResponse, Box<dyn Error>> {
    let conn = ctx.connection;
    Url::parse(&req.url)?;
    let namespace = MysqlNamespaces::find(conn, &req.namespace_id)?;
    let namespace = if let Some(namespace) = namespace {
        if !req.options.force {
            panic!(format!("namespace duplicated: {}", req.namespace_id));
        }
        MysqlNamespaces::update_and_find(
            conn,
            &req.namespace_id,
            &NamespaceUpdateForm {
                url: &req.url,
                description: req.options.description.as_ref().unwrap_or(&namespace.description),
                history_id: -1,
                version: -1,
                status: Status::DISABLED as i32,
                mtime: None,
                object_id: None,
                digest: None,
                size: None,
                fast_digest: None,
                updated_at: req.current_time.naive_utc(),
            },
        )?
    } else {
        MysqlNamespaces::insert_and_find(
            conn,
            &NamespaceInsertForm {
                id: &req.namespace_id,
                url: &req.url,
                description: req.options.description.as_ref().unwrap_or(&"".to_owned()),
                history_id: -1,
                version: -1,
                status: Status::DISABLED as i32,
                mtime: None,
                object_id: None,
                digest: None,
                size: None,
                fast_digest: None,
                created_at: req.current_time.naive_utc(),
                updated_at: req.current_time.naive_utc(),
            },
        )?
    };
    Ok(RegisterResponse { namespace })
}

#[derive(Debug)]
pub struct PullRequest<Tz: TimeZone> {
    pub namespace_id: String,
    pub current_time: DateTime<Tz>,
    pub options: PullOptions,
}

#[derive(Debug)]
pub struct PullOptions {}

impl Default for PullOptions {
    fn default() -> Self {
        PullOptions {}
    }
}

#[derive(Debug)]
pub struct PullResponse {
    pub namespace: Namespace,
}

pub fn pull<Tz: TimeZone>(ctx: &Context, req: &PullRequest<Tz>) -> Result<PullResponse, Box<dyn Error>> {
    let conn = ctx.connection;
    let namespace = MysqlNamespaces::find(conn, &req.namespace_id)?;
    let namespace = namespace.unwrap();
    let url = Url::parse(&namespace.url)?;
    let scheme = url.scheme();
    if scheme == "ssh" {
        let tempfile = ssh::download(&url)?;
        load_local_db(ctx, req, &namespace, tempfile.path())?;
        tempfile.close()?;
    } else if scheme == "file" {
        let path = Path::new(url.path());
        load_local_db(ctx, req, &namespace, path)?;
    } else {
        panic!(format!("unknown scheme: {}", scheme));
    }
    Ok(PullResponse { namespace })
}

fn load_local_db<Tz: TimeZone>(
    ctx: &Context,
    req: &PullRequest<Tz>,
    namespace: &Namespace,
    path: &Path,
) -> Result<(), Box<dyn Error>> {
    let global_conn = ctx.connection;
    let global_namespace_id = req.namespace_id.as_str();

    let meta_stat = MysqlStats::find_by_path(global_conn, META_NAMESPACE_ID, global_namespace_id)?;
    let updated_metadata = fs::new_updated_metadata_if_needed(&meta_stat, path)?;
    match updated_metadata {
        None => {
            return Ok(());
        }
        _ => {}
    }
    let updated_metadata = updated_metadata.unwrap();

    let local_conn = SqliteConnection::establish(path.to_str().unwrap())?;
    let local_conn = &local_conn;
    let local_namespace_id = ichno::consts::DEFAULT_NAMESPACE_ID;

    let local_stats = SqliteStats::select(&local_conn, local_namespace_id)?;
    for local_stat in local_stats.iter() {
        let path = &local_stat.path;
        let global_stat = MysqlStats::find_by_path(global_conn, global_namespace_id, path)?;
        if global_stat == None || global_stat.as_ref().unwrap().version != local_stat.version {
            let local_histories = SqliteHistories::select_by_path(local_conn, local_namespace_id, path)?;
            for local_history in local_histories.iter() {
                if let Some(global_stat) = global_stat.as_ref() {
                    if local_history.version <= global_stat.version {
                        continue;
                    }
                }
                let global_object = if let Some(local_object_id) = local_history.object_id {
                    let digest = local_history.digest.as_ref().unwrap();
                    let global_object = MysqlObjects::find_by_digest(global_conn, digest)?;
                    if let Some(_) = global_object {
                        global_object
                    } else {
                        let local_object = SqliteObjects::find(local_conn, local_object_id)?;
                        if let Some(local_object) = local_object {
                            Some(MysqlObjects::insert_and_find(
                                global_conn,
                                &ObjectInsertForm {
                                    digest: local_object.digest.as_str(),
                                    size: local_object.size,
                                    fast_digest: local_object.fast_digest,
                                    git_object_id: local_object.git_object_id.as_str(),
                                },
                            )?)
                        } else {
                            warn!("Object (id: {}) is not found in local DB", local_object_id);
                            None
                        }
                    }
                } else {
                    None
                };
                let global_history = MysqlHistories::insert_and_find(
                    global_conn,
                    &HistoryInsertForm {
                        namespace_id: global_namespace_id,
                        path,
                        version: local_history.version,
                        status: local_history.status,
                        mtime: local_history.mtime,
                        object_id: global_object.as_ref().map(|o| o.id),
                        digest: global_object.as_ref().map(|o| o.digest.as_str()),
                        created_at: local_history.created_at,
                        updated_at: local_history.updated_at,
                    },
                )?;
                if local_history.version == local_stat.version {
                    let _global_stat = if let Some(global_stat) = global_stat.as_ref() {
                        MysqlStats::update_and_find(
                            global_conn,
                            global_stat.id,
                            &StatUpdateForm {
                                history_id: global_history.id,
                                version: global_history.version,
                                status: global_history.status,
                                mtime: global_history.mtime,
                                object_id: global_history.object_id,
                                digest: global_object.as_ref().map(|o| o.digest.as_str()),
                                size: global_object.as_ref().map(|o| o.size),
                                fast_digest: global_object.as_ref().map(|o| o.fast_digest),
                                updated_at: local_stat.updated_at,
                            },
                        )?
                    } else {
                        MysqlStats::insert_and_find(
                            global_conn,
                            &StatInsertForm {
                                namespace_id: global_namespace_id,
                                path,
                                history_id: global_history.id,
                                version: global_history.version,
                                status: global_history.status,
                                mtime: global_history.mtime,
                                object_id: global_history.object_id,
                                digest: global_object.as_ref().map(|o| o.digest.clone()),
                                size: global_object.as_ref().map(|o| o.size),
                                fast_digest: global_object.as_ref().map(|o| o.fast_digest),
                                created_at: local_stat.created_at,
                                updated_at: local_stat.updated_at,
                            },
                        )?
                    };
                }
            }
        }
    }

    let stat = update_metadata(ctx, req, path, meta_stat.as_ref(), &updated_metadata)?;
    MysqlNamespaces::update(
        global_conn,
        global_namespace_id,
        &NamespaceUpdateForm {
            url: &namespace.url,
            description: &namespace.description,
            history_id: stat.history_id,
            version: stat.version,
            status: stat.status,
            mtime: stat.mtime,
            object_id: stat.object_id,
            digest: stat.digest.as_ref().map(|s| s.as_str()),
            size: stat.size,
            fast_digest: stat.fast_digest,
            updated_at: stat.updated_at,
        },
    )?;

    Ok(())
}

fn update_metadata<Tz: TimeZone>(
    ctx: &Context,
    req: &PullRequest<Tz>,
    path: &Path,
    meta_stat: Option<&Stat>,
    metadata: &FileMetadata,
) -> Result<Stat, Box<dyn Error>> {
    let conn = ctx.connection;
    let meta_namespace_id = META_NAMESPACE_ID;
    let global_namespace_id = &req.namespace_id;
    let now = req.current_time.naive_utc();
    let mut object = MysqlObjects::find_by_digest(conn, &metadata.digest)?;
    if object == None {
        let mut hasher = Sha1::default();
        treblo::object::blob_from_path(&mut hasher, path)?;
        let git_object_id = treblo::hex::to_hex_string(hasher.fixed_result().as_slice());
        object = Some(MysqlObjects::insert_and_find(
            conn,
            &ObjectInsertForm {
                digest: &metadata.digest,
                size: metadata.size,
                fast_digest: metadata.fast_digest,
                git_object_id: &git_object_id,
            },
        )?);
    }
    let object = object.unwrap();
    trace!("- object: {:?}", object);
    let last_history = MysqlHistories::find_latest_by_path(conn, meta_namespace_id, global_namespace_id)?;
    let version = match last_history {
        Some(h) => h.version + 1,
        None => 1,
    };
    let history = MysqlHistories::insert_and_find(
        conn,
        &HistoryInsertForm {
            namespace_id: meta_namespace_id,
            path: global_namespace_id,
            version,
            status: Status::ENABLED as i32,
            mtime: Some(metadata.mtime),
            object_id: Some(object.id),
            digest: Some(&object.digest),
            created_at: now,
            updated_at: now,
        },
    )?;
    trace!("- history: {:?}", history);
    let stat = match meta_stat {
        Some(old_stat) => MysqlStats::update_and_find(
            conn,
            old_stat.id,
            &StatUpdateForm {
                history_id: history.id,
                version: history.version,
                status: history.status,
                mtime: history.mtime,
                object_id: history.object_id,
                digest: Some(object.digest.as_str()),
                size: Some(object.size),
                fast_digest: Some(object.fast_digest),
                updated_at: now,
            },
        )?,
        None => MysqlStats::insert_and_find(
            conn,
            &StatInsertForm {
                namespace_id: meta_namespace_id,
                path: global_namespace_id,
                history_id: history.id,
                version: history.version,
                status: history.status,
                mtime: history.mtime,
                object_id: history.object_id,
                digest: Some(object.digest),
                size: Some(object.size),
                fast_digest: Some(object.fast_digest),
                created_at: now,
                updated_at: now,
            },
        )?,
    };
    trace!("- stat: {:?}", stat);
    Ok(stat)
}
