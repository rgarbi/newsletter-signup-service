use crate::auth::token::Claims;
use crate::db::subscribers_db_broker::retrieve_subscriber_by_id;
use actix_web::{web, HttpResponse, Responder};
use sqlx::PgPool;

use crate::db::subscriptions_db_broker::{
    retrieve_subscription_by_subscription_id, retrieve_subscriptions_by_subscriber_id,
};

use crate::util::from_path_to_uuid;

#[tracing::instrument(
    name = "Getting subscriptions by subscriber id",
    skip(id, pool, user),
    fields(
        id = %id,
    )
)]
pub async fn get_subscriptions_by_subscriber_id(
    id: web::Path<String>,
    pool: web::Data<PgPool>,
    user: Claims,
) -> impl Responder {
    match retrieve_subscriber_by_id(from_path_to_uuid(&id).unwrap(), &pool).await {
        Ok(subscriber) => {
            if subscriber.user_id != user.user_id {
                return HttpResponse::Unauthorized().finish();
            }
        }
        Err(_) => return HttpResponse::BadRequest().finish(),
    }

    match retrieve_subscriptions_by_subscriber_id(from_path_to_uuid(&id).unwrap(), &pool).await {
        Ok(subscriptions) => HttpResponse::Ok().json(subscriptions),
        Err(_) => HttpResponse::NotFound().finish(),
    }
}

#[tracing::instrument(
    name = "Getting subscriptions by subscriber id",
    skip(id, pool),
    fields(
        id = %id,
    )
)]
pub async fn get_subscription_by_id(
    id: web::Path<String>,
    pool: web::Data<PgPool>,
) -> impl Responder {
    match retrieve_subscription_by_subscription_id(from_path_to_uuid(&id).unwrap(), &pool).await {
        Ok(subscriptions) => HttpResponse::Ok().json(subscriptions),
        Err(_) => HttpResponse::NotFound().finish(),
    }
}
