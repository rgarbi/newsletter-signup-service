mod subscriber;
mod subscription;

#[macro_use]
extern crate rocket;

#[get("/")]
async fn index() -> &'static str {
    "Hello, world!"
}

#[get("/world")]
async fn world() -> &'static str {
    "hello, world!"
}

#[get("/<name>")]
async fn hello(name: &str) -> String {
    format!("Hello, {}!", name)
}

#[rocket::main]
async fn main() {
    rocket::build()
        .mount("/", routes![index])
        .mount("/hello", routes![world, hello])
        .launch()
        .await.unwrap();
}