use actix_web::{web, HttpResponse, Responder};
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::new_subscriber::{
    NewSubscriber, OverTheWireCreateSubscriber, OverTheWireSubscriber,
};
use crate::domain::subscriber_email::SubscriberEmail;
use crate::domain::subscriber_name::SubscriberName;

impl TryFrom<OverTheWireCreateSubscriber> for NewSubscriber {
    type Error = String;
    fn try_from(subscriber: OverTheWireCreateSubscriber) -> Result<Self, Self::Error> {
        let first_name = SubscriberName::parse(subscriber.first_name)?;
        let last_name = SubscriberName::parse(subscriber.last_name)?;
        let email_address = SubscriberEmail::parse(subscriber.email_address)?;
        Ok(NewSubscriber {
            first_name,
            last_name,
            email_address,
        })
    }
}

#[tracing::instrument(
    name = "Adding a new subscriber",
    skip(subscriber, pool),
    fields(
        subscriber_email = %subscriber.email_address,
    )
)]
pub async fn post_subscriber(
    subscriber: web::Form<OverTheWireCreateSubscriber>,
    pool: web::Data<PgPool>,
) -> impl Responder {
    let new_subscriber = match subscriber.0.try_into() {
        Ok(subscriber) => subscriber,
        Err(_) => return HttpResponse::BadRequest().finish(),
    };

    match insert_subscriber(&new_subscriber, &pool).await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[tracing::instrument(
name = "Getting a subscriber by email address",
skip(email, pool),
fields(
subscriber_email = %email,
)
)]
pub async fn get_subscriber_by_email(
    email: web::Path<String>,
    pool: web::Data<PgPool>,
) -> impl Responder {
    match retrieve_subscriber_by_email(email.into_inner(), &pool).await {
        Ok(subscriber) => HttpResponse::Ok().json(subscriber),
        Err(_) => HttpResponse::NotFound().finish(),
    }
}

#[tracing::instrument(
    name = "Saving new subscriber details in the database",
    skip(subscriber, pool)
)]
pub async fn insert_subscriber(
    subscriber: &NewSubscriber,
    pool: &PgPool,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"INSERT INTO subscribers (id, email_address, first_name, last_name) VALUES ($1, $2, $3, $4)"#,
        Uuid::new_v4(),
        subscriber.email_address.as_ref(),
        subscriber.first_name.as_ref(),
        subscriber.last_name.as_ref()
    ).execute(pool).await.map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;

    Ok(())
}

#[tracing::instrument(
    name = "Retrieving a subscriber from the database",
    skip(email_address, pool)
)]
pub async fn retrieve_subscriber_by_email(
    email_address: String,
    pool: &PgPool,
) -> Result<OverTheWireSubscriber, sqlx::Error> {
    let result = sqlx::query!(
        r#"SELECT id, email_address, first_name, last_name FROM subscribers WHERE  email_address = $1"#,
        email_address
    ).fetch_one(pool).await.map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;

    Ok(OverTheWireSubscriber {
        id: result.id.to_string(),
        last_name: result.last_name,
        email_address: result.email_address,
        first_name: result.first_name,
    })
}
