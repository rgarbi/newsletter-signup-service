#[macro_use]
extern crate rocket;

use rocket::fairing::{self, AdHoc};
use rocket::{Build, Rocket};

use rocket_db_pools::{sqlx, Connection, Database};

#[cfg(test)]
mod tests;

mod db;
mod models;
mod routes;

#[derive(Database)]
#[database("newsletter-signup")]
struct Db(sqlx::PgPool);

async fn run_migrations(rocket: Rocket<Build>) -> fairing::Result {
    match Db::fetch(&rocket) {
        Some(db) => match sqlx::migrate!("db/migrations").run(&**db).await {
            Ok(_) => Ok(rocket),
            Err(e) => {
                error!("Failed to initialize SQLx database: {}", e);
                Err(rocket)
            }
        },
        None => Err(rocket),
    }
}

pub fn stage() -> AdHoc {
    let database_url = "postgresql://...";
    let pool = sqlx::PgPool::connect(database_url)
        .await
        .expect("Failed to connect to database");
    AdHoc::on_ignite("SQLx Stage", |rocket| async {
        rocket
            .attach(AdHoc::try_on_ignite("SQLx Migrations", run_migrations))
            .manage(pool)
            .mount(
                "/subscribers",
                routes![routes::subscribers::get_subscriber_by_id],
            )
            .mount(
                "/subscriptions",
                routes![routes::subscriptions::find_subscription_by_id],
            )
    })
}

fn rocket() -> Rocket<Build> {
    rocket::build().attach(stage())
}

#[rocket::main]
async fn main() {
    rocket().launch().await.unwrap();
}
