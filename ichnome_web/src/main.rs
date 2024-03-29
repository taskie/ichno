#[macro_use]
extern crate actix_web;
#[macro_use]
extern crate log;

use std::{
    collections::{HashMap, HashSet},
    env,
};

use actix_web::{
    error, middleware,
    web::{self, Data},
    App, HttpResponse, HttpServer, Responder,
};
use chrono::NaiveDateTime;
use diesel::r2d2::{self, ConnectionManager};
use ichnome::{
    db::Connection,
    db::{OmFootprints, OmGroups, OmHistories, OmStats, OmWorkspaces, StatOrder, StatSearchCondition},
    error::DomainError,
    Footprint, Group, History, Stat, Status, Workspace, META_GROUP_NAME,
};
use serde::{Deserialize, Serialize};
use structopt::{clap, StructOpt};

use crate::models::{WebHistory, WebStat};

type DbPool = r2d2::Pool<ConnectionManager<Connection>>;

mod models;

fn find_workspace_and_group(
    conn: &mut Connection,
    workspace_name: &str,
    group_name: &str,
) -> Result<Option<(Workspace, Group)>, Box<dyn std::error::Error>> {
    let workspace = OmWorkspaces::find_by_name(conn, workspace_name)?;
    if let Some(workspace) = workspace {
        let group = OmGroups::find_by_name(conn, workspace.id, group_name)?;
        if let Some(group) = group {
            Ok(Some((workspace, group)))
        } else {
            Ok(None)
        }
    } else {
        Ok(None)
    }
}

#[derive(Deserialize)]
struct GetStatsQuery {
    path_prefix: Option<String>,
    path_partial: Option<String>,
    status: Option<String>,
    mtime_after: Option<NaiveDateTime>,
    mtime_before: Option<NaiveDateTime>,
    updated_at_after: Option<NaiveDateTime>,
    updated_at_before: Option<NaiveDateTime>,
    limit: Option<i64>,
}

#[derive(Serialize)]
struct GetStatsResponse {
    workspace: Workspace,
    group: Group,
    stats: Vec<Stat>,
    stats_count: i64,
}

fn get_stats_impl(
    conn: &mut Connection,
    workspace_name: &str,
    group_name: &str,
    q: &GetStatsQuery,
) -> Result<Option<GetStatsResponse>, Box<dyn std::error::Error>> {
    let pair = find_workspace_and_group(conn, workspace_name, group_name)?;
    if let Some((workspace, group)) = pair {
        let status = q
            .status
            .as_ref()
            .map(|s| match s.to_ascii_lowercase().as_str() {
                "0" | "disabled" => Some(Status::Disabled),
                "1" | "enabled" => Some(Status::Enabled),
                _ => None,
            })
            .flatten()
            .unwrap_or(Status::Enabled);
        let count_cond = StatSearchCondition {
            group_ids: Some(vec![group.id]),
            path_prefix: q.path_prefix.as_ref().map(|s| s.as_ref()),
            path_partial: q.path_partial.as_ref().map(|s| s.as_ref()),
            statuses: Some(vec![status]),
            mtime_after: q.mtime_after,
            mtime_before: q.mtime_before,
            updated_at_after: q.updated_at_after,
            updated_at_before: q.updated_at_before,
            ..Default::default()
        };
        let stats_count = OmStats::count(conn, workspace.id, &count_cond)?;
        let cond = StatSearchCondition {
            order: Some(StatOrder::UpdatedAtDesc),
            limit: Some(q.limit.unwrap_or(100)),
            ..count_cond
        };
        debug!("search condition: {:?}", &cond);
        let stats = OmStats::search(conn, workspace.id, &cond)?;
        Ok(Some(GetStatsResponse { workspace, group, stats, stats_count }))
    } else {
        Ok(None)
    }
}

#[get("/{workspace_name}/stats/{group_name}")]
async fn get_stats(
    pool: web::Data<DbPool>,
    path_params: web::Path<(String, String)>,
    q: web::Query<GetStatsQuery>,
) -> actix_web::Result<impl Responder> {
    let (workspace_name, group_name) = path_params.into_inner();
    let group_name_2 = group_name.clone();
    let q = q.into_inner();
    let mut conn = pool.get().expect("couldn't get db connection from pool");

    let resp =
        web::block(move || get_stats_impl(&mut conn, &workspace_name, &group_name, &q).map_err(|e| e.to_string()))
            .await?
            .map_err(error::ErrorInternalServerError)?;
    match resp {
        Some(resp) => Ok(HttpResponse::Ok().json(&resp)),
        None => {
            let res = HttpResponse::NotFound().body(format!("No group: {}", &group_name_2));
            Ok(res)
        }
    }
}

#[derive(Serialize)]
struct GetStatResponse {
    workspace: Workspace,
    group: Group,
    stat: Stat,
    histories: Option<Vec<History>>,
    footprints: Option<HashMap<i32, Footprint>>,
    eq_stats: Option<Vec<WebStat>>,
}

fn to_web_stats(workspace: &Workspace, group_map: &HashMap<i32, &Group>, stats: &Vec<Stat>) -> Vec<WebStat> {
    stats
        .iter()
        .map(|s| (s, group_map.get(&s.group_id)))
        .filter(|(_, g)| g.is_some())
        .map(|(s, g)| WebStat::from(&workspace, *g.unwrap(), s))
        .collect()
}

fn to_web_histories(workspace: &Workspace, group_map: &HashMap<i32, &Group>, stats: &Vec<History>) -> Vec<WebHistory> {
    stats
        .iter()
        .map(|h| (h, group_map.get(&h.group_id)))
        .filter(|(_, g)| g.is_some())
        .map(|(h, g)| WebHistory::from(&workspace, *g.unwrap(), h))
        .collect()
}

fn get_stat_impl(
    conn: &mut Connection,
    workspace_name: &str,
    group_name: &str,
    path: &str,
) -> Result<Option<GetStatResponse>, Box<dyn std::error::Error>> {
    let pair = find_workspace_and_group(conn, workspace_name, group_name)?;
    if let Some((workspace, group)) = pair {
        let stat = OmStats::find_by_path(conn, group.id, path)?;
        if let Some(stat) = stat {
            let histories = Some(OmHistories::select_by_path(conn, group.id, path)?);
            let footprints = Some({
                let mut footprint_ids = vec![];
                if let Some(footprint_id) = stat.footprint_id {
                    footprint_ids.push(footprint_id)
                }
                if let Some(histories) = histories.as_ref() {
                    for history in histories.iter() {
                        if let Some(footprint_id) = history.footprint_id {
                            footprint_ids.push(footprint_id);
                        }
                    }
                }
                let footprint_list = OmFootprints::select(conn, &footprint_ids)?;
                let mut footprints = HashMap::new();
                for footprint in footprint_list {
                    footprints.insert(footprint.id, footprint);
                }
                footprints
            });
            let eq_stats = Some({
                if let Some(footprint_id) = stat.footprint_id {
                    let stats = OmStats::select_by_footprint_id(conn, group.workspace_id, footprint_id)?;
                    let group_ids: Vec<i32> = stats.iter().map(|s| s.group_id).collect();
                    let groups = OmGroups::select(conn, &group_ids)?;
                    let group_map = groups.iter().map(|g| (g.id, g)).collect();
                    to_web_stats(&workspace, &group_map, &stats)
                } else {
                    vec![]
                }
            });
            Ok(Some(GetStatResponse { workspace, group, stat, histories, footprints, eq_stats }))
        } else {
            Ok(None)
        }
    } else {
        Ok(None)
    }
}

#[get("/{workspace_name}/stats/{group_name}/{path:.*}")]
async fn get_stat(
    pool: web::Data<DbPool>,
    path_params: web::Path<(String, String, String)>,
) -> actix_web::Result<impl Responder> {
    let (workspace_name, group_name, path) = path_params.into_inner();
    let group_name_2 = group_name.clone();
    let path_2 = path.clone();
    let mut conn = pool.get().expect("couldn't get db connection from pool");

    let resp =
        web::block(move || get_stat_impl(&mut conn, &workspace_name, &group_name, &path).map_err(|e| e.to_string()))
            .await?
            .map_err(error::ErrorInternalServerError)?;

    match resp {
        Some(resp) => Ok(HttpResponse::Ok().json(&resp)),
        None => {
            let res = HttpResponse::NotFound().body(format!("No stat: {}/{}", &group_name_2, &path_2));
            Ok(res)
        }
    }
}

#[derive(Serialize)]
struct GetFootprintResponse {
    workspace: Workspace,
    footprint: Footprint,
    group_name: Option<String>,
    stats: Option<Vec<WebStat>>,
    histories: Option<Vec<WebHistory>>,
}

fn get_footprint_impl(
    conn: &mut Connection,
    workspace_name: &str,
    digest: &str,
) -> Result<Option<GetFootprintResponse>, Box<dyn std::error::Error>> {
    let workspace = OmWorkspaces::find_by_name(conn, workspace_name)?;
    let workspace = if let Some(workspace) = workspace {
        workspace
    } else {
        return Ok(None);
    };
    let footprint = OmFootprints::find_by_digest(conn, digest)?;
    let group_name: Option<String> = None;
    if let Some(footprint) = footprint {
        let mut group_ids = HashSet::new();
        let stats = OmStats::select_by_footprint_id(conn, workspace.id, footprint.id)?;
        let histories = OmHistories::select_by_footprint_id(conn, workspace.id, footprint.id)?;
        if stats.is_empty() && histories.is_empty() {
            return Ok(None);
        }
        for s in stats.iter() {
            group_ids.insert(s.group_id);
        }
        for h in histories.iter() {
            group_ids.insert(h.group_id);
        }
        let group_ids: Vec<i32> = group_ids.into_iter().collect();
        let groups = OmGroups::select(conn, &group_ids)?;
        let group_map = groups.iter().map(|g| (g.id, g)).collect();
        let stats = to_web_stats(&workspace, &group_map, &stats);
        let histories = to_web_histories(&workspace, &group_map, &histories);
        Ok(Some(GetFootprintResponse {
            workspace,
            footprint,
            group_name,
            stats: Some(stats),
            histories: Some(histories),
        }))
    } else {
        Ok(None)
    }
}

#[get("/{workspace_name}/footprints/{digest}")]
async fn get_footprint(
    pool: web::Data<DbPool>,
    path_params: web::Path<(String, String)>,
) -> actix_web::Result<impl Responder> {
    let (workspace_name, digest) = path_params.into_inner();
    let digest_2 = digest.clone();
    let mut conn = pool.get().expect("couldn't get db connection from pool");

    let resp = web::block(move || get_footprint_impl(&mut conn, &workspace_name, &digest).map_err(|e| e.to_string()))
        .await?
        .map_err(error::ErrorInternalServerError)?;

    match resp {
        Some(resp) => Ok(HttpResponse::Ok().json(&resp)),
        None => {
            let res = HttpResponse::NotFound().body(format!("No footprint: {}", digest_2));
            Ok(res)
        }
    }
}

#[derive(Serialize)]
struct GetGroupsResponse {
    workspace: Workspace,
    groups: Vec<Group>,
}

fn get_groups_impl(
    conn: &mut Connection,
    workspace_name: &str,
) -> Result<Option<GetGroupsResponse>, Box<dyn std::error::Error>> {
    let workspace = OmWorkspaces::find_by_name(conn, workspace_name)?;
    if let Some(workspace) = workspace {
        let groups = OmGroups::select_all(conn, workspace.id)?;
        Ok(Some(GetGroupsResponse { workspace, groups }))
    } else {
        Ok(None)
    }
}

#[get("/{workspace_name}/groups")]
async fn get_groups(pool: web::Data<DbPool>, path_params: web::Path<(String,)>) -> actix_web::Result<impl Responder> {
    let (workspace_name,) = path_params.into_inner();
    let mut conn = pool.get().expect("couldn't get db connection from pool");

    let resp = web::block(move || get_groups_impl(&mut conn, &workspace_name).map_err(|e| e.to_string()))
        .await?
        .map_err(error::ErrorInternalServerError)?;

    match resp {
        Some(resp) => Ok(HttpResponse::Ok().json(&resp)),
        None => {
            let res = HttpResponse::NotFound().body("No groups");
            Ok(res)
        }
    }
}

#[derive(Serialize)]
struct GetGroupResponse {
    workspace: Workspace,
    group: Group,
    stat: Option<Stat>,
    histories: Option<Vec<History>>,
    footprints: Option<HashMap<i32, Footprint>>,
}

fn get_group_impl(
    conn: &mut Connection,
    workspace_name: &str,
    group_name: &str,
) -> Result<Option<GetGroupResponse>, Box<dyn std::error::Error>> {
    let pair = find_workspace_and_group(conn, workspace_name, group_name)?;
    if let Some((workspace, group)) = pair {
        let meta_group = OmGroups::find_by_name(conn, group.workspace_id, META_GROUP_NAME)?;
        let meta_group = if let Some(meta_group) = meta_group { meta_group } else { return Ok(None) };
        let stat = OmStats::find_by_path(conn, meta_group.id, group_name)?;
        let histories = Some(OmHistories::select_by_path(conn, meta_group.id, group_name)?);
        let footprints = Some({
            let mut footprint_ids = vec![];
            if let Some(stat) = stat.as_ref() {
                if let Some(footprint_id) = stat.footprint_id {
                    footprint_ids.push(footprint_id)
                }
            }
            if let Some(histories) = histories.as_ref() {
                for history in histories.iter() {
                    if let Some(footprint_id) = history.footprint_id {
                        footprint_ids.push(footprint_id);
                    }
                }
            }
            let footprint_list = OmFootprints::select(conn, &footprint_ids)?;
            let mut footprints = HashMap::new();
            for footprint in footprint_list {
                footprints.insert(footprint.id, footprint);
            }
            footprints
        });
        Ok(Some(GetGroupResponse { workspace, group, stat, histories, footprints }))
    } else {
        Ok(None)
    }
}

#[get("/{workspace_name}/groups/{group_name}")]
async fn get_group(
    pool: web::Data<DbPool>,
    path_params: web::Path<(String, String)>,
) -> actix_web::Result<impl Responder> {
    let (workspace_name, group_name) = path_params.into_inner();
    let group_name_2 = group_name.clone();
    let mut conn = pool.get().expect("couldn't get db connection from pool");

    let resp = web::block(move || get_group_impl(&mut conn, &workspace_name, &group_name).map_err(|e| e.to_string()))
        .await?
        .map_err(error::ErrorInternalServerError)?;

    match resp {
        Some(resp) => Ok(HttpResponse::Ok().json(&resp)),
        None => {
            let res = HttpResponse::NotFound().body(format!("No group: {}", &group_name_2));
            Ok(res)
        }
    }
}

#[derive(Deserialize)]
struct GetDiffQuery {
    group_name1: String,
    path_prefix1: String,
    group_name2: String,
    path_prefix2: String,
}

#[derive(Serialize)]
struct GetDiffResponse {
    workspace: Workspace,
    group1: Group,
    group2: Group,
    diff: HashMap<i32, (Vec<i32>, Vec<i32>)>,
    stats: HashMap<i32, Stat>,
    footprints: HashMap<i32, Footprint>,
}

fn get_diff_impl_search_stats(
    conn: &mut Connection,
    workspace: &Workspace,
    group_name: &str,
    path_prefix: &str,
) -> Result<Option<(Group, Vec<Stat>)>, Box<dyn std::error::Error>> {
    let group = OmGroups::find_by_name(conn, workspace.id, &group_name)?;
    let group = if let Some(group) = group { group } else { return Ok(None) };
    let cond =
        StatSearchCondition { group_ids: Some(vec![group.id]), path_prefix: Some(path_prefix), ..Default::default() };
    let stats_count = OmStats::count(conn, workspace.id, &cond)?;
    if stats_count > 1000 {
        return Err(Box::new(DomainError::params("path_prefix", format!("too many stats: {}", stats_count))));
    };
    let stats = OmStats::search(conn, workspace.id, &cond)?;
    Ok(Some((group, stats)))
}

fn get_diff_impl(
    conn: &mut Connection,
    workspace_name: &str,
    q: &GetDiffQuery,
) -> Result<Option<GetDiffResponse>, Box<dyn std::error::Error>> {
    let workspace = OmWorkspaces::find_by_name(conn, workspace_name)?;
    if let Some(workspace) = workspace {
        let result1 = get_diff_impl_search_stats(conn, &workspace, &q.group_name1, &q.path_prefix1)?;
        let (group1, stats1) = if let Some(x) = result1 { x } else { return Ok(None) };
        let result2 = get_diff_impl_search_stats(conn, &workspace, &q.group_name2, &q.path_prefix2)?;
        let (group2, stats2) = if let Some(x) = result2 { x } else { return Ok(None) };
        let stats: HashMap<i32, Stat> = stats1.iter().chain(stats2.iter()).map(|s| (s.id, s.clone())).collect();
        let mut diff = HashMap::<i32, (Vec<i32>, Vec<i32>)>::new();
        for stat1 in stats1.iter() {
            if let Some(footprint_id) = stat1.footprint_id {
                let v = diff.entry(footprint_id).or_insert_with(|| (vec![], vec![]));
                v.0.push(stat1.id);
            }
        }
        for stat2 in stats2.iter() {
            if let Some(footprint_id) = stat2.footprint_id {
                let v = diff.entry(footprint_id).or_insert_with(|| (vec![], vec![]));
                v.1.push(stat2.id);
            }
        }
        let footprints: HashMap<i32, Footprint> =
            OmFootprints::select(conn, &diff.keys().map(|i| *i).collect())?.into_iter().map(|f| (f.id, f)).collect();
        Ok(Some(GetDiffResponse { workspace, group1, group2, diff, stats, footprints }))
    } else {
        Ok(None)
    }
}

#[get("/{workspace_name}/diff")]
async fn get_diff(
    pool: web::Data<DbPool>,
    path_params: web::Path<(String,)>,
    q: web::Query<GetDiffQuery>,
) -> actix_web::Result<impl Responder> {
    let (workspace_name,) = path_params.into_inner();
    let q = q.into_inner();
    let mut conn = pool.get().expect("couldn't get db connection from pool");

    let resp = web::block(move || get_diff_impl(&mut conn, &workspace_name, &q).map_err(|e| e.to_string()))
        .await?
        .map_err(error::ErrorInternalServerError)?;

    match resp {
        Some(resp) => Ok(HttpResponse::Ok().json(&resp)),
        None => {
            let res = HttpResponse::NotFound().body(format!("Not found"));
            Ok(res)
        }
    }
}

#[derive(Debug, StructOpt)]
#[structopt(name = "ichnome-web")]
#[structopt(long_version(option_env!("LONG_VERSION").unwrap_or(env!("CARGO_PKG_VERSION"))))]
#[structopt(setting(clap::AppSettings::ColoredHelp))]
pub struct Opt {
    #[structopt(short, long, default_value = "127.0.0.1:3024")]
    pub address: String,
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    dotenv::dotenv().ok();
    let opt = Opt::from_args();

    let database_url = env::var("DATABASE_URL").unwrap();
    let manager = ConnectionManager::<Connection>::new(database_url);
    let pool = r2d2::Pool::builder().build(manager).expect("Failed to create pool.");

    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(pool.clone()))
            .wrap(middleware::Logger::default())
            .service(get_stats)
            .service(get_stat)
            .service(get_footprint)
            .service(get_groups)
            .service(get_group)
            .service(get_diff)
    })
    .bind(&opt.address)?
    .run()
    .await
}
