use sqlx::PgPool;

use rocket_db_pools::Connection;
use rocket_db_pools::{sqlx, Connection, Database};

use crate::models::subscriber::Subscriber;

#[derive(Database)]
#[database("newsletter-signup")]
struct Db(sqlx::PgPool);

pub async fn find_subscriber_by_id(id: i64) -> Option<Subscriber> {
    Option::from(Subscriber::new_subscriber(
        Option::from(id),
        String::from("first"),
        String::from("last"),
        String::from("first.last@somemail.com"),
    ))
}

pub async fn store_subscriber(pool: &rocket::State<PgPool>, subscriber: Subscriber) -> Subscriber {
    sqlx::query!(
        "INSERT INTO subscribers (first_name, last_name, email_address) VALUES (?, ?, ?)",
        subscriber.first_name,
        subscriber.last_name,
        subsriber.email_address
    )
    .execute(pool.inner())
    .await;

    subscriber
}
