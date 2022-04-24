#[macro_use]
extern crate rocket;
extern crate openssl;
#[macro_use]
extern crate diesel;
extern crate dotenv;
#[macro_use]
extern crate diesel_migrations;
embed_migrations!("./migrations");

use std::env;

use dotenv::dotenv;
use model::model::get_api;
use rocket::{
    fairing::AdHoc,
    figment::{
        map,
        value::{Map, Value},
    },
    fs::FileServer,
    Build, Rocket,
};
use rocket_sync_db_pools::database;

pub mod model;
pub mod models;
pub mod schema;
pub mod util;

#[database("db")]
pub struct Database(diesel::PgConnection);

#[launch]
fn rocket() -> _ {
    dotenv();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL is missing");
    let db: Map<_, Value> = map! {
        "url" => database_url.into(),
    };

    let figment = rocket::Config::figment().merge(("databases", map!["db" => db]));

    rocket::custom(figment)
        .attach(Database::fairing())
        .attach(AdHoc::try_on_ignite(
            "Database Migrations",
            run_db_migrations,
        ))
        .mount("/api/v1", get_api())
        .mount("/", FileServer::from("./static"))
}

async fn run_db_migrations(rocket: Rocket<Build>) -> Result<Rocket<Build>, Rocket<Build>> {
    let db = Database::get_one(&rocket)
        .await
        .expect("Unable to open database connection for migration");
    db.run(|conn| match embedded_migrations::run(&*conn) {
        Ok(()) => Ok(rocket),
        Err(e) => {
            error!("Failed to run database migrations: {:?}", e);
            Err(rocket)
        }
    })
    .await
}
