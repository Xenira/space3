#[macro_use]
extern crate rocket;
#[macro_use]
extern crate diesel;
extern crate dotenv;
#[macro_use]
extern crate diesel_migrations;
embed_migrations!("./migrations");

use dotenv::dotenv;
use model::model::get_api;
use rocket::{
    fairing::AdHoc,
    fs::{FileServer, NamedFile},
    Build, Rocket,
};
use rocket_sync_db_pools::database;

pub mod model;
pub mod models;
pub mod schema;
pub mod util;

#[database("db")]
pub struct Database(diesel::PgConnection);

#[get("/<_..>", rank = 30)]
async fn index() -> Result<NamedFile, std::io::Error> {
    NamedFile::open("static/index.html").await
}

#[launch]
fn rocket() -> _ {
    dotenv();

    rocket::build()
        .attach(Database::fairing())
        .attach(AdHoc::try_on_ignite(
            "Database Migrations",
            run_db_migrations,
        ))
        .mount("/api/v1", get_api())
        .mount("/static", FileServer::from("./static"))
        .mount("/", routes![index])
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
