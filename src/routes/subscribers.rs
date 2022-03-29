use std::fmt::Debug;

use actix_web::{web, HttpResponse, Responder};
use serde::Deserialize;
use serde_json::json;
use sqlx::PgPool;

use crate::auth::token::Claims;
use crate::db::subscribers::{
    insert_subscriber, retrieve_subscriber_by_email, retrieve_subscriber_by_id,
};
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
            user_id: String::new(),
        })
    }
}

#[tracing::instrument(
    name = "Adding a new subscriber",
    skip(subscriber, pool, user),
    fields(
        subscriber_email = %subscriber.email_address,
    )
)]
pub async fn post_subscriber(
    subscriber: web::Json<OverTheWireCreateSubscriber>,
    pool: web::Data<PgPool>,
    user: Claims,
) -> impl Responder {
    let new_subscriber: NewSubscriber = match subscriber.0.try_into() {
        Ok(subscriber) => subscriber,
        Err(_) => return HttpResponse::BadRequest().finish(),
    };

    if new_subscriber.user_id.clone() != user.user_id {
        return HttpResponse::Unauthorized().finish();
    }

    let mut transaction = match pool.begin().await {
        Ok(transaction) => transaction,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };

    match insert_subscriber(&new_subscriber, &mut transaction).await {
        Ok(_) => HttpResponse::Ok().json(json!({})),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[tracing::instrument(
    name = "Getting a subscriber by email address",
    skip(email_query, pool, user),
    fields(
        subscriber_email = %email_query.email,
    )
)]
pub async fn get_subscriber_by_email(
    email_query: web::Query<EmailAddressQuery>,
    pool: web::Data<PgPool>,
    user: Claims,
) -> impl Responder {
    match retrieve_subscriber_by_email(email_query.0.email, &pool).await {
        Ok(subscriber) => check_user_is_the_owner_of_this_record(user, subscriber),
        Err(_) => HttpResponse::NotFound().finish(),
    }
}

#[tracing::instrument(
    name = "Getting a subscriber by id",
    skip(id, pool, user),
    fields(
        id = %id,
    )
)]
pub async fn get_subscriber_by_id(
    id: web::Path<String>,
    pool: web::Data<PgPool>,
    user: Claims,
) -> impl Responder {
    match retrieve_subscriber_by_id(from_string_to_uuid(id).unwrap(), &pool).await {
        Ok(subscriber) => check_user_is_the_owner_of_this_record(user, subscriber),
        Err(_) => HttpResponse::NotFound().finish(),
    }
}

pub fn check_user_is_the_owner_of_this_record(
    user: Claims,
    subscriber: OverTheWireSubscriber,
) -> HttpResponse {
    if subscriber.user_id == user.user_id {
        return HttpResponse::Ok().json(subscriber);
    }
    tracing::error!(
        "A user with id: {} does not have access to a user with id {}",
        user.user_id,
        subscriber.user_id
    );
    HttpResponse::Unauthorized().finish()
}
