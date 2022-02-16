use rocket::response::status::Created;
use rocket::serde::json::Json;
use rocket_db_pools::{sqlx, Connection, Database};

use crate::db::subscribers::find_subscriber_by_id;
use crate::models::subscriber::Subscriber;
use sqlx::PgPool;

#[derive(Database)]
#[database("newsletter-signup")]
struct Db(sqlx::PgPool);

type Result<T, E = rocket::response::Debug<sqlx::Error>> = std::result::Result<T, E>;

#[get("/<id>")]
pub async fn get_subscriber_by_id(id: i32) -> Json<Subscriber> {
    Json(find_subscriber_by_id(id).await.unwrap())
}

#[post("/", data = "<subscriber>")]
pub async fn create_subscriber(
    pool: &rocket::State<PgPool>,
    subscriber: Json<Subscriber>,
) -> Result<Created<Json<Subscriber>>> {
    Ok(Created::new("/").body(create_subscriber(db, subscriber)))
}
