
#[get("/<id>")]
pub async fn get_by_id(id: i32) -> String {
    format!("id, {}!", id)
}