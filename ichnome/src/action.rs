use std::{error::Error, path::Path};

use chrono::{DateTime, NaiveDateTime, Utc};
use diesel::{Connection, MysqlConnection, SqliteConnection};
use ichno::db::{SqliteFootprints, SqliteGroups, SqliteHistories, SqliteStats, SqliteWorkspaces};

use url::Url;

use crate::{
    constants::GroupType,
    db::{
        actions::{
            create_footprint_if_needed, create_group_if_needed, create_meta_group_if_needed,
            create_workspace_if_needed, new_updated_file_state_if_needed, update_meta_group_stat, FileState,
        },
        MysqlFootprints, MysqlGroups, MysqlHistories, MysqlStats, MysqlWorkspaces,
    },
    models::{
        Group, GroupUpdateForm, HistoryInsertForm, StatInsertForm, StatUpdateForm, Workspace, WorkspaceUpdateForm,
    },
    ssh,
};

pub struct Context<'c> {
    pub connection: &'c MysqlConnection,
    pub timer: Box<dyn Fn() -> DateTime<Utc>>,
}

impl<'c> Context<'c> {
    pub fn current_time(&self) -> DateTime<Utc> {
        (self.timer)()
    }

    pub fn naive_current_time(&self) -> NaiveDateTime {
        self.current_time().naive_utc()
    }
}

#[derive(Debug)]
pub struct SetupRequest {
    pub workspace_name: String,
    pub options: SetupOptions,
}

#[derive(Default, Debug)]
pub struct SetupOptions {
    pub description: Option<String>,
    pub force: bool,
}

#[derive(Debug)]
pub struct SetupResponse {
    pub workspace: Workspace,
}

pub fn setup(ctx: &Context, req: &SetupRequest) -> Result<SetupResponse, Box<dyn Error>> {
    let conn = ctx.connection;
    let now = ctx.naive_current_time();
    let workspace = MysqlWorkspaces::find_by_name(conn, &req.workspace_name)?;
    if let Some(workspace) = workspace {
        if req.options.force {
            MysqlWorkspaces::update_and_find(
                conn,
                workspace.id,
                &WorkspaceUpdateForm {
                    description: req.options.description.as_ref().map(|s| s.as_str()),
                    ..Default::default()
                },
            )?;
        } else {
            panic!("workspae {} already exists", workspace.name)
        }
    }
    let workspace = create_workspace_if_needed(conn, &req.workspace_name, now)?;
    Ok(SetupResponse { workspace })
}

#[derive(Debug)]
pub struct RegisterRequest {
    pub workspace_name: String,
    pub group_name: String,
    pub url: String,
    pub options: RegisterOptions,
}

#[derive(Default, Debug)]
pub struct RegisterOptions {
    pub description: Option<String>,
    pub force: bool,
}

#[derive(Debug)]
pub struct RegisterResponse {
    pub workspace: Workspace,
    pub group: Group,
}

pub fn register(ctx: &Context, req: &RegisterRequest) -> Result<RegisterResponse, Box<dyn Error>> {
    let conn = ctx.connection;
    let now = ctx.naive_current_time();
    let workspace = MysqlWorkspaces::find_by_name(conn, &req.workspace_name)?.unwrap();
    let url = Url::parse(&req.url)?;
    let group = MysqlGroups::find_by_name(conn, workspace.id, &req.group_name)?;
    if let Some(group) = group {
        if req.options.force {
            MysqlGroups::update_and_find(
                conn,
                group.id,
                &GroupUpdateForm {
                    url: Some(&req.url),
                    description: req.options.description.as_ref().map(|s| s.as_str()),
                    ..Default::default()
                },
            )?;
        } else {
            panic!("group {} already exists", group.name)
        }
    }
    let group = create_group_if_needed(conn, &workspace, &req.group_name, &url, GroupType::Remote, now)?;
    Ok(RegisterResponse { workspace, group })
}

#[derive(Debug)]
pub struct PullRequest {
    pub workspace_name: String,
    pub group_name: String,
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
    pub group: Group,
}

pub fn pull(ctx: &Context, req: &PullRequest) -> Result<PullResponse, Box<dyn Error>> {
    let conn = ctx.connection;
    let workspace = MysqlWorkspaces::find_by_name(conn, &req.workspace_name)?.unwrap();
    let group = MysqlGroups::find_by_name(conn, workspace.id, &req.group_name)?.unwrap();
    let url = Url::parse(&group.url)?;
    let scheme = url.scheme();
    if scheme == "ssh" {
        let tempfile = ssh::download(&url)?;
        load_local_db(ctx, req, &workspace, &group, tempfile.path())?;
        tempfile.close()?;
    } else if scheme == "file" {
        let path = Path::new(url.path());
        load_local_db(ctx, req, &workspace, &group, path)?;
    } else {
        panic!(format!("unknown scheme: {}", scheme));
    }
    Ok(PullResponse { group })
}

fn load_local_db(
    ctx: &Context,
    _req: &PullRequest,
    glb_workspace: &Workspace,
    glb_group: &Group,
    path: &Path,
) -> Result<(), Box<dyn Error>> {
    let glb_conn = ctx.connection;
    let now = ctx.naive_current_time();
    let meta_group = create_meta_group_if_needed(glb_conn, glb_workspace, now)?;
    let meta_stat = MysqlStats::find_by_path(glb_conn, meta_group.id, &glb_group.name)?;
    let _updated_metadata = if let Some(FileState::Enabled(updated_metadata)) =
        new_updated_file_state_if_needed(meta_stat.as_ref(), path)?
    {
        updated_metadata
    } else {
        return Ok(());
    };

    let loc_conn = SqliteConnection::establish(path.to_str().unwrap())?;
    let loc_conn = &loc_conn;
    let loc_workspace_name = ichno::DEFAULT_WORKSPACE_NAME;
    let loc_workspace = SqliteWorkspaces::find_by_name(loc_conn, loc_workspace_name)?.unwrap();
    let loc_group_name = ichno::DEFAULT_GROUP_NAME;
    let loc_group = SqliteGroups::find_by_name(loc_conn, loc_workspace.id, loc_group_name)?.unwrap();

    let loc_stats = SqliteStats::select_by_group_id(loc_conn, loc_group.id)?;
    for loc_stat in loc_stats.iter() {
        let path = &loc_stat.path;
        let glb_stat = MysqlStats::find_by_path(glb_conn, glb_group.id, path)?;
        if glb_stat == None || glb_stat.as_ref().unwrap().version != loc_stat.version {
            let loc_histories = SqliteHistories::select_by_path(loc_conn, loc_group.id, path)?;
            for loc_history in loc_histories.iter() {
                if let Some(glb_stat) = glb_stat.as_ref() {
                    if loc_history.version <= glb_stat.version {
                        continue;
                    }
                }
                let glb_footprint = if let Some(loc_footprint_id) = loc_history.footprint_id {
                    let digest = loc_history.digest.as_ref().unwrap();
                    let glb_footprint = MysqlFootprints::find_by_digest(glb_conn, digest)?;
                    if let Some(_) = glb_footprint {
                        glb_footprint
                    } else {
                        let loc_footprint = SqliteFootprints::find(loc_conn, loc_footprint_id)?;
                        if let Some(loc_footprint) = loc_footprint {
                            Some(create_footprint_if_needed(
                                glb_conn,
                                loc_footprint.digest.as_str(),
                                loc_footprint.size,
                                loc_footprint.fast_digest,
                                now,
                            )?)
                        } else {
                            warn!("Footprint (id: {}) is not found in local DB", loc_footprint_id);
                            None
                        }
                    }
                } else {
                    None
                };
                let glb_history = MysqlHistories::insert_and_find(
                    glb_conn,
                    &HistoryInsertForm {
                        workspace_id: glb_workspace.id,
                        group_id: glb_group.id,
                        path,
                        version: loc_history.version,
                        status: loc_history.status,
                        mtime: loc_history.mtime,
                        footprint_id: glb_footprint.as_ref().map(|o| o.id),
                        digest: glb_footprint.as_ref().map(|o| o.digest.as_str()),
                        created_at: loc_history.created_at,
                        updated_at: loc_history.updated_at,
                    },
                )?;
                if loc_history.version == loc_stat.version {
                    let _glb_stat = if let Some(glb_stat) = glb_stat.as_ref() {
                        MysqlStats::update_and_find(
                            glb_conn,
                            glb_stat.id,
                            &StatUpdateForm {
                                history_id: Some(glb_history.id),
                                version: Some(glb_history.version),
                                status: Some(glb_history.status),
                                mtime: Some(glb_history.mtime),
                                footprint_id: Some(glb_history.footprint_id),
                                digest: Some(glb_footprint.as_ref().map(|o| o.digest.as_str())),
                                size: Some(glb_footprint.as_ref().map(|o| o.size)),
                                fast_digest: Some(glb_footprint.as_ref().map(|o| o.fast_digest)),
                                updated_at: Some(loc_stat.updated_at),
                            },
                        )?
                    } else {
                        MysqlStats::insert_and_find(
                            glb_conn,
                            &StatInsertForm {
                                workspace_id: glb_workspace.id,
                                group_id: glb_group.id,
                                path,
                                history_id: glb_history.id,
                                version: glb_history.version,
                                status: glb_history.status,
                                mtime: glb_history.mtime,
                                footprint_id: glb_history.footprint_id,
                                digest: glb_footprint.as_ref().map(|o| o.digest.as_str()),
                                size: glb_footprint.as_ref().map(|o| o.size),
                                fast_digest: glb_footprint.as_ref().map(|o| o.fast_digest),
                                created_at: loc_stat.created_at,
                                updated_at: loc_stat.updated_at,
                            },
                        )?
                    };
                }
            }
        }
    }

    let _group = update_meta_group_stat(glb_conn, glb_workspace, glb_group, path, now)?;

    Ok(())
}
