use rocket::serde::json::Json;

use crate::db::subscriptions::find_subscriptions_by_id;
use crate::models::subscription::Subscription;

#[get("/<id>")]
pub async fn find_subscription_by_id(id: i32) -> Json<Subscription> {
    Json(find_subscriptions_by_id(id).await.unwrap())
}
