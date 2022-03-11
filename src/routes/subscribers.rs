use std::fmt::{Debug, Display};

use actix_web::{web, HttpResponse, Responder, ResponseError};
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::new_subscriber::{
    NewSubscriber, OverTheWireCreateSubscriber, OverTheWireSubscriber,
};
use crate::domain::valid_email::ValidEmail;
use crate::domain::valid_name::ValidName;
use crate::util::from_string_to_uuid;

#[derive(Debug, Deserialize)]
pub struct EmailAddressQuery {
    email: String,
}

impl TryFrom<OverTheWireCreateSubscriber> for NewSubscriber {
    type Error = String;
    fn try_from(subscriber: OverTheWireCreateSubscriber) -> Result<Self, Self::Error> {
        let first_name = ValidName::parse(subscriber.first_name)?;
        let last_name = ValidName::parse(subscriber.last_name)?;
        let email_address = ValidEmail::parse(subscriber.email_address)?;
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
    subscriber: web::Json<OverTheWireCreateSubscriber>,
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
    skip(email_query, pool),
    fields(
        subscriber_email = %email_query.email,
    )
)]
pub async fn get_subscriber_by_email(
    email_query: web::Query<EmailAddressQuery>,
    pool: web::Data<PgPool>,
) -> impl Responder {
    match retrieve_subscriber_by_email(email_query.0.email, &pool).await {
        Ok(subscriber) => HttpResponse::Ok().json(subscriber),
        Err(_) => HttpResponse::NotFound().finish(),
    }
}

#[tracing::instrument(
    name = "Getting a subscriber by id",
    skip(id, pool),
    fields(
        id = %id,
    )
)]
pub async fn get_subscriber_by_id(
    id: web::Path<String>,
    pool: web::Data<PgPool>,
) -> impl Responder {
    match retrieve_subscriber_by_id(from_string_to_uuid(id).unwrap(), &pool).await {
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
) -> Result<(), StoreSubscriberError> {
    sqlx::query!(
        r#"INSERT INTO subscribers (id, email_address, first_name, last_name) VALUES ($1, $2, $3, $4)"#,
        Uuid::new_v4(),
        subscriber.email_address.as_ref(),
        subscriber.first_name.as_ref(),
        subscriber.last_name.as_ref()
    ).execute(pool).await.map_err(|e| {
        let err = StoreSubscriberError(e);
        tracing::error!("{:?}", err);
        err

    })?;

    Ok(())
}

#[tracing::instrument(
    name = "Retrieving a subscriber by email address from the database",
    skip(email_address, pool)
)]
pub async fn retrieve_subscriber_by_email(
    email_address: String,
    pool: &PgPool,
) -> Result<OverTheWireSubscriber, sqlx::Error> {
    let result = sqlx::query!(
        r#"SELECT id, email_address, first_name, last_name FROM subscribers WHERE email_address = $1"#,
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

#[tracing::instrument(
    name = "Retrieving a subscriber by id from the database",
    skip(id, pool)
)]
pub async fn retrieve_subscriber_by_id(
    id: Uuid,
    pool: &PgPool,
) -> Result<OverTheWireSubscriber, sqlx::Error> {
    let result = sqlx::query!(
        r#"SELECT id, email_address, first_name, last_name FROM subscribers WHERE id = $1"#,
        id
    )
    .fetch_one(pool)
    .await
    .map_err(|e| {
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

#[derive(Debug)]
pub struct StoreSubscriberError(sqlx::Error);

impl Display for StoreSubscriberError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "A database error was encountered while trying to store a subscriber."
        )
    }
}

impl ResponseError for StoreSubscriberError {}
