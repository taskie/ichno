#[macro_use]
extern crate actix_web;
#[macro_use]
extern crate log;

use std::{collections::HashMap, env};

use actix_web::{middleware, web, App, Error, HttpResponse, HttpServer, Responder};
use diesel::{
    r2d2::{self, ConnectionManager},
    MysqlConnection,
};
use ichnome::{
    db::{MysqlHistories, MysqlNamespaces, MysqlObjects, MysqlStats},
    History, Namespace, Object, Stat, META_NAMESPACE_ID,
};
use serde::Serialize;
use structopt::{clap, StructOpt};

type DbPool = r2d2::Pool<ConnectionManager<MysqlConnection>>;

#[derive(Serialize)]
struct GetStatsResponse {
    namespace: Namespace,
    stats: Vec<Stat>,
}

fn get_stats_impl(
    conn: &MysqlConnection,
    namespace_id: &str,
) -> Result<Option<GetStatsResponse>, Box<dyn std::error::Error>> {
    let namespace = MysqlNamespaces::find(conn, namespace_id)?;
    match namespace {
        Some(namespace) => {
            let stats = MysqlStats::select_by_namespace_id(conn, namespace_id)?;
            Ok(Some(GetStatsResponse { namespace, stats }))
        }
        None => Ok(None),
    }
}

#[get("/stats/{namespace_id}")]
async fn get_stats(pool: web::Data<DbPool>, path_params: web::Path<String>) -> Result<impl Responder, Error> {
    let namespace_id = path_params.into_inner();
    let namespace_id_2 = namespace_id.clone();
    let conn = pool.get().expect("couldn't get db connection from pool");

    let resp =
        web::block(move || get_stats_impl(&conn, &namespace_id).map_err(|e| e.to_string())).await.map_err(|e| {
            error!("{}", e);
            HttpResponse::InternalServerError().finish()
        })?;
    match resp {
        Some(resp) => Ok(HttpResponse::Ok().json(&resp)),
        None => {
            let res = HttpResponse::NotFound().body(format!("No namespace: {}", &namespace_id_2));
            Ok(res)
        }
    }
}

#[derive(Serialize)]
struct GetStatResponse {
    namespace: Namespace,
    stat: Stat,
    histories: Option<Vec<History>>,
    objects: Option<HashMap<i32, Object>>,
    eq_stats: Option<Vec<Stat>>,
}

fn get_stat_impl(
    conn: &MysqlConnection,
    namespace_id: &str,
    path: &str,
) -> Result<Option<GetStatResponse>, Box<dyn std::error::Error>> {
    let namespace = MysqlNamespaces::find(conn, namespace_id).map_err(|e| e.to_string())?;
    if let Some(namespace) = namespace {
        let stat = MysqlStats::find_by_path(conn, namespace_id, path)?;
        if let Some(stat) = stat {
            let histories = Some(MysqlHistories::select_by_path(conn, namespace_id, path)?);
            let objects = Some({
                let mut object_ids = vec![];
                if let Some(object_id) = stat.object_id {
                    object_ids.push(object_id)
                }
                if let Some(histories) = histories.as_ref() {
                    for history in histories.iter() {
                        if let Some(object_id) = history.object_id {
                            object_ids.push(object_id);
                        }
                    }
                }
                let object_list = MysqlObjects::select(conn, &object_ids)?;
                let mut objects = HashMap::new();
                for object in object_list {
                    objects.insert(object.id, object);
                }
                objects
            });
            let eq_stats = Some({
                if let Some(object_id) = stat.object_id {
                    MysqlStats::select_by_object_id(conn, None, object_id)?
                } else {
                    vec![]
                }
            });
            Ok(Some(GetStatResponse { namespace, stat, histories, objects, eq_stats }))
        } else {
            Ok(None)
        }
    } else {
        Ok(None)
    }
}

#[get("/stats/{namespace_id}/{path:.*}")]
async fn get_stat(pool: web::Data<DbPool>, path_params: web::Path<(String, String)>) -> Result<impl Responder, Error> {
    let (namespace_id, path) = path_params.into_inner();
    let namespace_id_2 = namespace_id.clone();
    let path_2 = path.clone();
    let conn = pool.get().expect("couldn't get db connection from pool");

    let resp = web::block(move || get_stat_impl(&conn, &namespace_id, &path).map_err(|e| e.to_string()))
        .await
        .map_err(|e| {
            error!("{}", e);
            HttpResponse::InternalServerError().finish()
        })?;

    match resp {
        Some(resp) => Ok(HttpResponse::Ok().json(&resp)),
        None => {
            let res = HttpResponse::NotFound().body(format!("No stat: {}/{}", &namespace_id_2, &path_2));
            Ok(res)
        }
    }
}

#[derive(Serialize)]
struct GetObjectResponse {
    object: Object,
    namespace_id: Option<String>,
    stats: Option<Vec<Stat>>,
    histories: Option<Vec<History>>,
}

fn get_object_impl(
    conn: &MysqlConnection,
    digest: &str,
) -> Result<Option<GetObjectResponse>, Box<dyn std::error::Error>> {
    let object = MysqlObjects::find_by_digest(conn, digest)?;
    let namespace_id: Option<String> = None;
    if let Some(object) = object {
        let stats = Some(MysqlStats::select_by_object_id(conn, namespace_id.as_ref().map(|s| s.as_str()), object.id)?);
        let histories =
            Some(MysqlHistories::select_by_object_id(conn, namespace_id.as_ref().map(|s| s.as_str()), object.id)?);
        Ok(Some(GetObjectResponse { object, namespace_id, stats, histories }))
    } else {
        Ok(None)
    }
}

#[get("/objects/{digest}")]
async fn get_object(pool: web::Data<DbPool>, path_params: web::Path<String>) -> Result<impl Responder, Error> {
    let digest = path_params.into_inner();
    let digest_2 = digest.clone();
    let conn = pool.get().expect("couldn't get db connection from pool");

    let resp = web::block(move || get_object_impl(&conn, &digest).map_err(|e| e.to_string())).await.map_err(|e| {
        error!("{}", e);
        HttpResponse::InternalServerError().finish()
    })?;

    match resp {
        Some(resp) => Ok(HttpResponse::Ok().json(&resp)),
        None => {
            let res = HttpResponse::NotFound().body(format!("No object: {}", digest_2));
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

#[derive(Serialize)]
struct GetNamespacesResponse {
    namespaces: Vec<Namespace>,
}

fn get_namespaces_impl(conn: &MysqlConnection) -> Result<Option<GetNamespacesResponse>, Box<dyn std::error::Error>> {
    let namespaces = MysqlNamespaces::select_all(conn)?;
    Ok(Some(GetNamespacesResponse { namespaces }))
}

#[get("/namespaces")]
async fn get_namespaces(pool: web::Data<DbPool>) -> Result<impl Responder, Error> {
    let conn = pool.get().expect("couldn't get db connection from pool");

    let resp = web::block(move || get_namespaces_impl(&conn).map_err(|e| e.to_string())).await.map_err(|e| {
        error!("{}", e);
        HttpResponse::InternalServerError().finish()
    })?;
    match resp {
        Some(resp) => Ok(HttpResponse::Ok().json(&resp)),
        None => {
            let res = HttpResponse::NotFound().body("No namespaces");
            Ok(res)
        }
    }
}

#[derive(Serialize)]
struct GetNamespaceResponse {
    namespace: Namespace,
    stat: Option<Stat>,
    histories: Option<Vec<History>>,
    objects: Option<HashMap<i32, Object>>,
}

fn get_namespace_impl(
    conn: &MysqlConnection,
    namespace_id: &str,
) -> Result<Option<GetNamespaceResponse>, Box<dyn std::error::Error>> {
    let namespace = MysqlNamespaces::find(conn, namespace_id)?;
    if let Some(namespace) = namespace {
        let stat = MysqlStats::find_by_path(conn, META_NAMESPACE_ID, namespace_id)?;
        let histories = Some(MysqlHistories::select_by_path(conn, META_NAMESPACE_ID, namespace_id)?);
        let objects = Some({
            let mut object_ids = vec![];
            if let Some(stat) = stat.as_ref() {
                if let Some(object_id) = stat.object_id {
                    object_ids.push(object_id)
                }
            }
            if let Some(histories) = histories.as_ref() {
                for history in histories.iter() {
                    if let Some(object_id) = history.object_id {
                        object_ids.push(object_id);
                    }
                }
            }
            let object_list = MysqlObjects::select(conn, &object_ids)?;
            let mut objects = HashMap::new();
            for object in object_list {
                objects.insert(object.id, object);
            }
            objects
        });
        Ok(Some(GetNamespaceResponse { namespace, stat, histories, objects }))
    } else {
        Ok(None)
    }
}

#[get("/namespaces/{namespace_id}")]
async fn get_namespace(pool: web::Data<DbPool>, path_params: web::Path<String>) -> Result<impl Responder, Error> {
    let namespace_id = path_params.into_inner();
    let namespace_id_2 = namespace_id.clone();
    let conn = pool.get().expect("couldn't get db connection from pool");

    let resp =
        web::block(move || get_namespace_impl(&conn, &namespace_id).map_err(|e| e.to_string())).await.map_err(|e| {
            error!("{}", e);
            HttpResponse::InternalServerError().finish()
        })?;
    match resp {
        Some(resp) => Ok(HttpResponse::Ok().json(&resp)),
        None => {
            let res = HttpResponse::NotFound().body(format!("No namespace: {}", &namespace_id_2));
            Ok(res)
        }
    }
}
#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    dotenv::dotenv().ok();
    let opt = Opt::from_args();

    let database_url = env::var("DATABASE_URL").unwrap();
    let manager = ConnectionManager::<MysqlConnection>::new(database_url);
    let pool = r2d2::Pool::builder().build(manager).expect("Failed to create pool.");

    HttpServer::new(move || {
        App::new()
            .data(pool.clone())
            .wrap(middleware::Logger::default())
            .service(get_stats)
            .service(get_stat)
            .service(get_object)
            .service(get_namespaces)
            .service(get_namespace)
    })
    .bind(&opt.address)?
    .run()
    .await
}
