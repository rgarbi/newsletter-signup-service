use crate::auth::token::Claims;
use crate::db::subscribers_db_broker::retrieve_subscriber_by_id;
use actix_web::{web, HttpResponse, Responder};
use sqlx::PgPool;
use uuid::Uuid;

use crate::db::subscriptions_db_broker::{
    cancel_subscription_by_subscription_id, retrieve_subscription_by_subscription_id,
    retrieve_subscriptions_by_subscriber_id,
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
    name = "Getting subscription by subscription id",
    skip(id, pool, user),
    fields(
        id = %id,
    )
)]
pub async fn get_subscription_by_id(
    id: web::Path<String>,
    pool: web::Data<PgPool>,
    user: Claims,
) -> impl Responder {
    match retrieve_subscription_by_subscription_id(from_path_to_uuid(&id).unwrap(), &pool).await {
        Ok(subscription) => {
            match reject_unauthorized_user(subscription.subscriber_id, user.user_id, &pool).await {
                Ok(_) => {}
                Err(response) => return response,
            };
            HttpResponse::Ok().json(subscription)
        }
        Err(_) => HttpResponse::NotFound().finish(),
    }
}

#[tracing::instrument(
    name = "Cancel subscription by subscription id",
    skip(id, pool, user),
    fields(
        id = %id,
    )
)]
pub async fn cancel_subscription_by_id(
    id: web::Path<String>,
    pool: web::Data<PgPool>,
    user: Claims,
) -> impl Responder {
    let subscription_id = from_path_to_uuid(&id).unwrap();
    match retrieve_subscription_by_subscription_id(subscription_id.clone(), &pool).await {
        Ok(subscription) => {
            match reject_unauthorized_user(subscription.subscriber_id, user.user_id, &pool).await {
                Ok(_) => {}
                Err(response) => return response,
            };

            let mut transaction = match pool.begin().await {
                Ok(transaction) => transaction,
                Err(_) => return HttpResponse::InternalServerError().finish(),
            };

            //Set it to active = false
            match cancel_subscription_by_subscription_id(subscription_id.clone(), &mut transaction)
                .await
            {
                Ok(_) => {}
                Err(_) => {
                    transaction.rollback().await.unwrap();
                    return HttpResponse::InternalServerError().finish();
                }
            }

            //Add a history object....
            //Call stripe to cancel the subscription

            HttpResponse::Ok().json(subscription)
        }
        Err(_) => HttpResponse::NotFound().finish(),
    }
}

async fn reject_unauthorized_user(
    subscriber_id: Uuid,
    user_id: String,
    pool: &PgPool,
) -> Result<(), HttpResponse> {
    return match retrieve_subscriber_by_id(subscriber_id, &pool).await {
        Ok(subscriber) => {
            if subscriber.user_id != user_id {
                return Err(HttpResponse::Unauthorized().finish());
            }
            Ok(())
        }
        Err(_) => Err(HttpResponse::BadRequest().finish()),
    };
}
