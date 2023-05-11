#[macro_use]
extern crate rocket;
extern crate openssl;
#[macro_use]
extern crate diesel;
extern crate dotenv;

use diesel::{pg::Pg, PgConnection};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use scheduler::long_running_task;
pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("./migrations");

use std::{env, error::Error, sync::Mutex};

use dotenv::dotenv;
use model::{model::get_api, polling::ActivePolls};
use rocket::{
    fairing::AdHoc,
    figment::{
        map,
        value::{Map, Value},
    },
    fs::FileServer,
    tokio, Build, Rocket,
};
use rocket_sync_db_pools::database;

pub mod model;
pub mod models;
pub(crate) mod scheduler;
pub mod schema;
pub(crate) mod service;
pub mod util;

#[database("db")]
pub struct Database(diesel::PgConnection);

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    if let Err(err) = dotenv() {
        warn!("Failed to read dotenv: {}", err);
    }

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL is missing");
    let pool_size: u32 = env::var("POOL_SIZE").map_or(10, |size| {
        size.parse()
            .expect(format!("POOL_SIZE {} can not be cast to u32", size).as_str())
    });
    let port: u32 = env::var("PORT").map_or(8000, |port| {
        port.parse()
            .expect(format!("PORT {} can not be cast to u32", port).as_str())
    });

    let db: Map<_, Value> = map! {
        "url" => database_url.into(),
        "pool_size" => pool_size.into()
    };

    let figment = rocket::Config::figment()
        .merge(("port", port))
        .merge(("databases", map!["db" => db]));

    let r = rocket::custom(figment)
        .attach(Database::fairing())
        .attach(AdHoc::try_on_ignite(
            "Database Migrations",
            run_db_migrations,
        ))
        .mount("/api/v1", get_api())
        .mount("/", FileServer::from("./static"))
        .ignite()
        .await?;

    let conn = Database::get_one(&r).await.unwrap();

    tokio::spawn(async move { long_running_task(conn).await });

    r.launch().await?;

    Ok(())
}

async fn run_db_migrations(rocket: Rocket<Build>) -> Result<Rocket<Build>, Rocket<Build>> {
    let db = Database::get_one(&rocket)
        .await
        .expect("Unable to open database connection for migration");

    db.run(|conn| match run_migrations(conn) {
        Ok(()) => Ok(rocket),
        Err(e) => {
            error!("Failed to run database migrations: {:?}", e);
            Err(rocket)
        }
    })
    .await
}

fn run_migrations(
    connection: &mut impl MigrationHarness<Pg>,
) -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    // This will run the necessary migrations.
    //
    // See the documentation for `MigrationHarness` for
    // all available methods.
    debug!("Running migrations");
    connection.run_pending_migrations(MIGRATIONS)?;

    Ok(())
}
