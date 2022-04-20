use std::fmt::Debug;

use actix_web::{web, HttpResponse, Responder};
use serde::Deserialize;
use serde_json::json;
use sqlx::PgPool;

use crate::auth::token::Claims;
use crate::db::subscribers::{
    insert_subscriber, retrieve_subscriber_by_email, retrieve_subscriber_by_id,
    retrieve_subscriber_by_user_id, retrieve_subscriber_by_user_id_and_email_address,
};
use crate::domain::subscriber_models::{
    NewSubscriber, OverTheWireCreateSubscriber, OverTheWireSubscriber,
};
use crate::domain::valid_email::ValidEmail;
use crate::domain::valid_name::ValidName;
use crate::util::from_string_to_uuid;

#[derive(Debug, Deserialize)]
pub struct SubscriberQueries {
    user_id: Option<String>,
    email: Option<String>,
}

impl TryFrom<OverTheWireCreateSubscriber> for NewSubscriber {
    type Error = String;
    fn try_from(subscriber: OverTheWireCreateSubscriber) -> Result<Self, Self::Error> {
        let name = ValidName::parse(subscriber.name)?;
        let email_address = ValidEmail::parse(subscriber.email_address)?;
        Ok(NewSubscriber {
            name,
            email_address,
            user_id: subscriber.user_id,
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
        Ok(_) => {
            if transaction.commit().await.is_err() {
                HttpResponse::InternalServerError().finish();
            }
            HttpResponse::Ok().json(json!({}))
        }
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[tracing::instrument(
    name = "Getting a subscriber either by email address or by user id",
    skip(query, pool, user)
)]
pub async fn get_subscriber_by_query(
    query: web::Query<SubscriberQueries>,
    pool: web::Data<PgPool>,
    user: Claims,
) -> impl Responder {
    if query.user_id.is_none() && query.email.is_none() {
        return HttpResponse::NotFound().finish();
    }

    if query.user_id.is_some() && query.email.is_some() {
        return get_subscriber_by_email_and_user_id(
            query.0.email.unwrap(),
            query.0.user_id.unwrap(),
            pool,
            user,
        )
        .await;
    }

    if query.user_id.is_some() && query.email.is_none() {
        return get_subscriber_by_user_id(query.0.user_id.unwrap(), pool, user).await;
    }

    if query.user_id.is_none() && query.email.is_some() {
        return get_subscriber_by_email(query.0.email.unwrap(), pool, user).await;
    }

    tracing::error!("Could not handle the query params given. {:?}", query);
    HttpResponse::InternalServerError().finish()
}

#[tracing::instrument(
    name = "Getting a subscriber by email address",
    skip(email, pool, user),
    fields(
        subscriber_email = %email,
    )
)]
async fn get_subscriber_by_email(
    email: String,
    pool: web::Data<PgPool>,
    user: Claims,
) -> HttpResponse {
    match retrieve_subscriber_by_email(email, &pool).await {
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
    match retrieve_subscriber_by_id(from_string_to_uuid(&id).unwrap(), &pool).await {
        Ok(subscriber) => check_user_is_the_owner_of_this_record(user, subscriber),
        Err(_) => HttpResponse::NotFound().finish(),
    }
}

#[tracing::instrument(
    name = "Getting a subscriber by user id",
    skip(user_id, pool, user),
    fields(
        user_id = %user_id,
    )
)]
async fn get_subscriber_by_user_id(
    user_id: String,
    pool: web::Data<PgPool>,
    user: Claims,
) -> HttpResponse {
    match retrieve_subscriber_by_user_id(user_id.as_str(), &pool).await {
        Ok(subscriber) => check_user_is_the_owner_of_this_record(user, subscriber),
        Err(_) => HttpResponse::NotFound().finish(),
    }
}

#[tracing::instrument(
    name = "Getting a subscriber by email address",
    skip(email, user_id, pool, user),
    fields(
        subscriber_email = %email,
        subscriber_user_id = %user_id
    )
)]
async fn get_subscriber_by_email_and_user_id(
    email: String,
    user_id: String,
    pool: web::Data<PgPool>,
    user: Claims,
) -> HttpResponse {
    match retrieve_subscriber_by_user_id_and_email_address(user_id.as_str(), email.as_str(), &pool)
        .await
    {
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
