use crate::subscriber::model::Subscriber;
use rocket::serde::json::Json;


#[get("/<id>")]
pub async fn get_by_id(id: i32) -> Json<Subscriber> {
    Json(Subscriber::get_subscriber_by_id(id).await.unwrap())
}