use std::str::FromStr;

use actix_web::{web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use tracing::Instrument;
use uuid::Uuid;

#[derive(Deserialize, Serialize)]
pub struct Subscriber {
    pub id: String,
    pub first_name: String,
    pub last_name: String,
    pub email_address: String,
}

pub async fn subscriber(
    subscriber: web::Form<Subscriber>,
    pool: web::Data<PgPool>,
) -> impl Responder {
    let request_id = Uuid::new_v4();
    let request_span = tracing::info_span!("Adding a new subscriber.", %request_id, subscriber_email = %subscriber.email_address);
    let _request_span_guard = request_span.enter();

    let query_span = tracing::info_span!("Saving new subscriber details in the database");
    let result = sqlx::query!(
        r#"INSERT INTO subscribers (id, email_address, first_name, last_name) VALUES ($1, $2, $3, $4)"#,
        Uuid::from_str(&*subscriber.id).expect("Unable to parse the UUID"),
        subscriber.email_address,
        subscriber.first_name,
        subscriber.last_name
    ).execute(pool.get_ref()).instrument(query_span).await;

    match result {
        Ok(_) => {
            tracing::info!("New subscriber details have been saved");
            HttpResponse::Ok().finish()
        }
        Err(e) => {
            tracing::error!(
                "request id {} - Failed to execute query: {:?}",
                request_id,
                e
            );
            HttpResponse::InternalServerError().finish()
        }
    }
}
