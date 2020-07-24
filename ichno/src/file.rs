use std::{
    convert::AsRef,
    error::Error,
    fs::File,
    hash::Hasher,
    io::{Read, Seek, SeekFrom, Write},
    path::{Path, PathBuf},
    time::UNIX_EPOCH,
};

use chrono::{DateTime, NaiveDateTime, TimeZone};
use diesel::sqlite::SqliteConnection;
use sha1::Sha1;
use sha2::{digest::FixedOutput, Sha256};
use twox_hash::XxHash64;
use url::Url;

use crate::{
    constants::{Status, META_NAMESPACE_ID},
    db::{SqliteHistories, SqliteNamespaces, SqliteObjects, SqliteStats},
    models::{
        HistoryInsertForm, Namespace, NamespaceInsertForm, NamespaceUpdateForm, ObjectInsertForm, Stat, StatInsertForm,
        StatUpdateForm,
    },
};

#[derive(Debug)]
pub(crate) struct FileMetadata {
    pub size: i64,
    pub mtime: NaiveDateTime,
    pub fast_digest: i64,
    pub digest: String,
}

pub(crate) fn new_updated_metadata_if_needed(
    stat: &Option<Stat>,
    path: &Path,
) -> Result<Option<FileMetadata>, Box<dyn Error>> {
    let mut f = File::open(path)?;
    let mut should_check_fast_digest = false;
    let md = f.metadata()?;
    let modified_unix = md.modified()?.duration_since(UNIX_EPOCH)?;
    let mtime = NaiveDateTime::from_timestamp(modified_unix.as_secs() as i64, modified_unix.subsec_nanos());
    if let Some(stat) = stat {
        if let Some(old_mtime) = stat.mtime {
            if mtime != old_mtime {
                should_check_fast_digest = true;
            }
        }
    }
    let size = md.len() as i64;
    if let Some(stat) = stat {
        if let Some(old_size) = stat.size {
            if size != old_size {
                should_check_fast_digest = true;
            }
        }
    }
    if let Some(_) = stat {
        if !should_check_fast_digest {
            return Ok(None);
        }
    }
    let mut buf = [0u8; 8096];
    let fast_digest = {
        let mut fast_hasher = XxHash64::default();
        loop {
            let n = f.read(&mut buf)?;
            if n == 0 {
                break;
            }
            Hasher::write(&mut fast_hasher, &buf[0..n]);
        }
        Hasher::finish(&fast_hasher) as i64
    };
    if let Some(stat) = stat {
        if let Some(old_fast_digest) = stat.fast_digest {
            if fast_digest == old_fast_digest {
                return Ok(None);
            }
        }
    }
    f.seek(SeekFrom::Start(0))?;
    let digest = {
        let mut hasher = Sha256::default();
        loop {
            let n = f.read(&mut buf)?;
            if n == 0 {
                break;
            }
            hasher.write(&buf[0..n])?;
        }
        treblo::hex::to_hex_string(hasher.fixed_result().as_slice())
    };
    if let Some(stat) = stat {
        if let Some(old_digest) = stat.digest.clone() {
            if digest == old_digest {
                return Ok(None);
            }
        }
    }
    Ok(Some(FileMetadata { size, mtime, fast_digest, digest }))
}

pub struct Context<'c, 'a, Tz: TimeZone> {
    pub connection: &'c SqliteConnection,
    pub db_path: &'a Path,
    pub namespace_id: &'a str,
    pub namespace: Option<Namespace>,
    pub current_time: DateTime<Tz>,
}

impl<'c, 'a, Tz: TimeZone> Context<'c, 'a, Tz> {
    pub fn naive_current_time(&self) -> NaiveDateTime {
        self.current_time.naive_utc()
    }

    pub fn base_directory(&self) -> Option<PathBuf> {
        self.namespace.as_ref().and_then(|ns| Url::parse(&ns.url).ok()).map(|url| PathBuf::from(url.path()))
    }
}

pub fn pre_process<Tz: TimeZone>(ctx: &mut Context<Tz>) -> Result<(), Box<dyn Error>> {
    let conn = ctx.connection;
    let namespace_id = ctx.namespace_id;
    let now = ctx.naive_current_time();
    let mut namespace = SqliteNamespaces::find(conn, namespace_id)?;
    if let Some(namespace) = namespace {
        ctx.namespace = Some(namespace);
    } else {
        let abs_db_path = ctx.db_path.canonicalize()?;
        let base_dir = abs_db_path.parent().unwrap();
        let url = format!("file://{}", base_dir.to_str().unwrap());
        namespace = Some(SqliteNamespaces::insert_and_find(
            &conn,
            &NamespaceInsertForm {
                id: namespace_id,
                url: &url,
                description: "",
                history_id: -1,
                version: -1,
                status: Status::DISABLED as i32,
                mtime: None,
                object_id: None,
                digest: None,
                size: None,
                fast_digest: None,
                created_at: now,
                updated_at: now,
            },
        )?);
        trace!("- namespace: {:?}", namespace.as_ref().unwrap());
        ctx.namespace = Some(namespace.unwrap().clone());
    }
    Ok(())
}

pub fn post_process<Tz: TimeZone>(ctx: &mut Context<Tz>) -> Result<(), Box<dyn Error>> {
    let meta_namespace_id = META_NAMESPACE_ID;
    let stat = upsert_with_file(ctx, meta_namespace_id, ctx.db_path)?;
    let old_namespace = ctx.namespace.clone().unwrap();
    let namespace = SqliteNamespaces::update_and_find(
        ctx.connection,
        ctx.namespace_id,
        &NamespaceUpdateForm {
            url: &old_namespace.url,
            description: &old_namespace.description,
            history_id: stat.history_id,
            version: stat.version,
            status: stat.status,
            mtime: stat.mtime,
            object_id: stat.object_id,
            digest: stat.digest.as_ref().map(|s| s.as_ref()),
            size: stat.size,
            fast_digest: stat.fast_digest,
            updated_at: ctx.naive_current_time(),
        },
    )?;
    trace!("- namespace: {:?}", namespace);
    ctx.namespace = Some(namespace);
    Ok(())
}

pub fn remove_with_file<Tz: TimeZone, P: AsRef<Path>>(
    ctx: &Context<Tz>,
    namespace_id: &str,
    path: P,
) -> Result<Option<Stat>, Box<dyn Error>> {
    let conn = ctx.connection;
    let base_path = ctx.base_directory().unwrap();
    let path = base_path.join(path);
    let path_ref = path.strip_prefix(base_path)?;
    let path_str = path_ref.to_str().expect(&format!("invalid path string"));
    debug!("* {}", path_str);
    let old_stat = SqliteStats::find_by_path(conn, namespace_id, path_str)?;
    if let Some(old_stat) = old_stat {
        if old_stat.status == Status::DISABLED as i32 {
            return Ok(Some(old_stat));
        }
        let now = ctx.naive_current_time();
        let last_history = SqliteHistories::find_latest_by_path(conn, namespace_id, path_str)?;
        let version = match last_history {
            Some(h) => h.version + 1,
            None => 1,
        };
        let history = SqliteHistories::insert_and_find(
            conn,
            &HistoryInsertForm {
                namespace_id,
                path: path_str,
                version,
                status: Status::DISABLED as i32,
                mtime: None,
                object_id: None,
                digest: None,
                created_at: now,
                updated_at: now,
            },
        )?;
        trace!("- history: {:?}", history);
        let stat = SqliteStats::update_and_find(
            conn,
            old_stat.id,
            &StatUpdateForm {
                history_id: history.id,
                version: history.version,
                status: history.status,
                mtime: history.mtime,
                object_id: history.object_id,
                digest: None,
                size: None,
                fast_digest: None,
                updated_at: now,
            },
        )?;
        trace!("- stat: {:?}", stat);
        Ok(Some(stat))
    } else {
        Ok(None)
    }
}

pub fn upsert_with_file<Tz: TimeZone, P: AsRef<Path>>(
    ctx: &Context<Tz>,
    namespace_id: &str,
    path: P,
) -> Result<Stat, Box<dyn Error>> {
    let conn = ctx.connection;
    let base_path = ctx.base_directory().unwrap();
    let path = path.as_ref().canonicalize()?;
    let path_ref = path.strip_prefix(base_path)?;
    let path_str = path_ref.to_str().expect(&format!("invalid path string"));
    debug!("* {}", path_str);
    let old_stat = SqliteStats::find_by_path(conn, namespace_id, path_str)?;
    let now = ctx.naive_current_time();
    let metadata = new_updated_metadata_if_needed(&old_stat, path_ref)?;
    trace!("- updated_metadata: {:?}", metadata);
    if let Some(metadata) = metadata {
        let mut object = SqliteObjects::find_by_digest(conn, &metadata.digest)?;
        if object == None {
            let mut hasher = Sha1::default();
            treblo::object::blob_from_path(&mut hasher, path_ref)?;
            let git_object_id = treblo::hex::to_hex_string(hasher.fixed_result().as_slice());
            object = Some(SqliteObjects::insert_and_find(
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
        let last_history = SqliteHistories::find_latest_by_path(conn, namespace_id, path_str)?;
        let version = match last_history {
            Some(h) => h.version + 1,
            None => 1,
        };
        let history = SqliteHistories::insert_and_find(
            conn,
            &HistoryInsertForm {
                namespace_id,
                path: path_str,
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
        let stat = match old_stat {
            Some(old_stat) => SqliteStats::update_and_find(
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
            None => SqliteStats::insert_and_find(
                conn,
                &StatInsertForm {
                    namespace_id,
                    path: path_str,
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
    } else {
        Ok(old_stat.unwrap())
    }
}
