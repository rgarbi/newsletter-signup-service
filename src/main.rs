#[macro_use]
extern crate rocket;

mod db;
mod models;
mod routes;
mod subscription;

#[get("/")]
async fn index() -> &'static str {
    "Hello, world!"
}

#[rocket::main]
async fn main() {
    rocket::build()
        .mount("/", routes![index])
        .mount("/subscribers", routes![routes::subscribers::get_by_id])
        .launch()
        .await
        .unwrap();
}
