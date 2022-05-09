use crate::auth::token::Claims;
use crate::background::subscription_history_storer::store_subscription_history_event;
use crate::configuration::get_configuration;
use crate::db::subscribers_db_broker::retrieve_subscriber_by_id;
use actix_web::{web, HttpResponse, Responder};
use reqwest::Error;
use secrecy::ExposeSecret;
use serde_json::json;
use sqlx::PgPool;
use tracing::Level;
use uuid::Uuid;

use crate::db::subscriptions_db_broker::{
    cancel_subscription_by_subscription_id, retrieve_subscription_by_subscription_id,
    retrieve_subscriptions_by_subscriber_id,
};
use crate::domain::subscription_history_models::HistoryEventType;

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
    let config = get_configuration().unwrap();
    match retrieve_subscription_by_subscription_id(subscription_id, &pool).await {
        Ok(subscription) => {
            match reject_unauthorized_user(subscription.subscriber_id, user.user_id, &pool).await {
                Ok(_) => {}
                Err(response) => return response,
            };

            if !subscription.active {
                tracing::event!(Level::INFO, "This subscription has already been cancelled!");
                return HttpResponse::Ok().json(json!({}));
            }

            let mut transaction = match pool.begin().await {
                Ok(transaction) => transaction,
                Err(_) => return HttpResponse::InternalServerError().finish(),
            };

            //Set it to active = false
            match cancel_subscription_by_subscription_id(subscription_id, &mut transaction).await {
                Ok(_) => {}
                Err(_) => {
                    transaction.rollback().await.unwrap();
                    return HttpResponse::InternalServerError().finish();
                }
            }

            //Call stripe to cancel the subscription
            match cancel_stripe_subscription(
                subscription.stripe_subscription_id,
                config
                    .stripe_client
                    .api_secret_key
                    .expose_secret()
                    .to_string(),
            )
            .await
            {
                Ok(_) => {
                    if transaction.commit().await.is_err() {
                        HttpResponse::InternalServerError().finish();
                    }
                    //Add a history object....
                    store_subscription_history_event(
                        subscription.id,
                        HistoryEventType::Cancelled,
                        &pool,
                    );
                    HttpResponse::Ok().json(json!({}))
                }
                Err(_) => {
                    transaction.rollback().await.unwrap();
                    HttpResponse::InternalServerError().finish()
                }
            }
        }
        Err(_) => HttpResponse::NotFound().finish(),
    }
}

async fn reject_unauthorized_user(
    subscriber_id: Uuid,
    user_id: String,
    pool: &PgPool,
) -> Result<(), HttpResponse> {
    return match retrieve_subscriber_by_id(subscriber_id, pool).await {
        Ok(subscriber) => {
            if subscriber.user_id != user_id {
                return Err(HttpResponse::Unauthorized().finish());
            }
            Ok(())
        }
        Err(_) => Err(HttpResponse::BadRequest().finish()),
    };
}

async fn cancel_stripe_subscription(
    subscription_id: String,
    stripe_publishable_key: String,
) -> Result<(), Error> {
    let response = reqwest::Client::new()
        .delete(format!(
            "https://api.stripe.com/v1/subscriptions/{}",
            subscription_id
        ))
        .basic_auth(stripe_publishable_key, Option::Some(String::new()))
        .send()
        .await;

    return match response {
        Ok(response) => {
            let response_body = response.text().await.unwrap();
            tracing::event!(Level::INFO, "Got the following back!! {:?}", &response_body);
            Ok(())
        }
        Err(err) => {
            tracing::event!(Level::ERROR, "Err: {:?}", err);
            Err(err)
        }
    };
}
