#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;
#[macro_use]
extern crate log;

use std::{env, error::Error, ffi::OsStr};

use chrono::Local;
use diesel::{connection::Connection, sqlite::SqliteConnection};
use dotenv;
use ignore;
use std::{path::Path, process::exit};

use crate::consts::DEFAULT_NAMESPACE_ID;

pub mod consts;
pub mod fs;
pub mod models;
pub mod schema;
pub mod sqlite;

embed_migrations!("migrations");

fn main_with_error() -> Result<i32, Box<dyn Error>> {
    dotenv::dotenv().ok();
    env_logger::init();

    let database_url = env::var("DATABASE_URL").unwrap_or("ichno.db".to_owned());
    let conn = SqliteConnection::establish(&database_url).expect(&format!("Error connecting to {}", database_url));
    let db_path = Path::new(&database_url).canonicalize()?;

    embedded_migrations::run(&conn).unwrap();

    let namespace_id = DEFAULT_NAMESPACE_ID;
    let mut ctx =
        fs::Context { connection: &conn, db_path: &db_path, namespace_id, namespace: None, current_time: Local::now() };
    let path = Path::new(".");
    let abs_path = path.canonicalize()?;

    let w = {
        let mut wb = ignore::WalkBuilder::new(&abs_path);
        wb.filter_entry(|p| p.file_name() != OsStr::new(".git") && p.file_name() != OsStr::new("ichno.db"));
        wb.build()
    };
    fs::pre_process(&mut ctx)?;
    for result in w {
        match result {
            Ok(entry) => {
                if entry.metadata().unwrap().is_file() {
                    fs::upsert_with_file(&ctx, namespace_id, entry.path())?;
                }
            }
            Err(err) => warn!("{}", err),
        }
    }
    fs::post_process(&mut ctx)?;
    Ok(0)
}

fn main() {
    match main_with_error() {
        Ok(code) => exit(code),
        Err(e) => error!("{}", e),
    }
}
