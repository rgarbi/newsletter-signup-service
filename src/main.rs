mod subscriber;
mod subscription;

#[macro_use]
extern crate rocket;

#[get("/")]
async fn index() -> &'static str {
    "Hello, world!"
}

#[rocket::main]
async fn main() {

    rocket::build()
        .mount("/", routes![index])
        .mount("/subscribers", routes![subscriber::handlers::get_by_id])
        .launch()
        .await.unwrap();
}