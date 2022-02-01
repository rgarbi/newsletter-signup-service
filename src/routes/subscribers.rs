use rocket::serde::json::Json;

use crate::db::subscribers::find_subscriber_by_id;
use crate::models::subscriber::Subscriber;

#[get("/<id>")]
pub async fn get_by_id(id: i32) -> Json<Subscriber> {
    Json(find_subscriber_by_id(id).await.unwrap())
}
