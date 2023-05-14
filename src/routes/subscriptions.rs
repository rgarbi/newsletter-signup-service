use crate::auth::token::Claims;
use crate::background::subscription_history_storer::store_subscription_history_event;
use crate::db::subscribers_db_broker::retrieve_subscriber_by_id;
use actix_web::{web, HttpResponse, Responder};
use chrono::{Datelike, DateTime, NaiveDateTime, Utc};
use serde_json::json;
use sqlx::PgPool;
use tracing::Level;
use uuid::Uuid;

use crate::db::subscriptions_db_broker::{
    cancel_subscription_by_subscription_id, retrieve_subscription_by_subscription_id,
    retrieve_subscriptions_by_subscriber_id, update_subscription_by_subscription_id,
};
use crate::domain::subscription_history_models::HistoryEventType;
use crate::domain::subscription_models::OverTheWireSubscription;
use crate::domain::valid_email::ValidEmail;
use crate::domain::valid_name::ValidName;
use crate::stripe_client::StripeClient;

use crate::util::from_path_to_uuid;

#[tracing::instrument(
name = "Getting subscriptions by subscriber id",
skip(id, pool, user),
fields(
id = % id,
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
id = % id,
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
name = "Updating subscription",
skip(id, subscription, pool, user),
fields(
id = % id,
)
)]
pub async fn update_subscription(
    id: web::Path<String>,
    subscription: web::Json<OverTheWireSubscription>,
    pool: web::Data<PgPool>,
    user: Claims,
) -> impl Responder {
    match retrieve_subscription_by_subscription_id(from_path_to_uuid(&id).unwrap(), &pool).await {
        Ok(stored_subscription) => {
            match reject_unauthorized_user(subscription.subscriber_id, user.user_id, &pool).await {
                Ok(_) => {}
                Err(response) => return response,
            };

            let valid_name = ValidName::parse(subscription.subscription_name.clone());
            let valid_email = ValidEmail::parse(subscription.subscription_email_address.clone());

            if valid_email.is_err()
                || valid_name.is_err()
                || stored_subscription.id != subscription.id
            {
                return HttpResponse::BadRequest().finish();
            }

            match update_subscription_by_subscription_id(
                stored_subscription.id,
                subscription.0,
                &pool,
            )
                .await
            {
                Ok(_) => HttpResponse::Ok().json(json!({})),
                Err(_) => HttpResponse::InternalServerError().finish(),
            }
        }
        Err(_) => HttpResponse::NotFound().finish(),
    }
}

#[tracing::instrument(
name = "Cancel subscription by subscription id",
skip(id, pool, user, stripe_client),
fields(
id = % id,
)
)]
pub async fn cancel_subscription_by_id(
    id: web::Path<String>,
    pool: web::Data<PgPool>,
    user: Claims,
    stripe_client: web::Data<StripeClient>,
) -> impl Responder {
    let subscription_id = from_path_to_uuid(&id).unwrap();
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
            match stripe_client
                .cancel_stripe_subscription(subscription.stripe_subscription_id)
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
    match retrieve_subscriber_by_id(subscriber_id, pool).await {
        Ok(subscriber) => {
            if subscriber.user_id != user_id {
                return Err(HttpResponse::Unauthorized().finish());
            }
            Ok(())
        }
        Err(_) => Err(HttpResponse::BadRequest().finish()),
    }
}

async fn calculate_renewal_year(renewal_month: u32, renewal_day: u32, subscription_creation_date: DateTime<Utc>) -> u32 {
    let current_day = Utc::now().day();
    let current_month = Utc::now().month();
    let current_year = Utc::now().year();



    //same day
    if current_month == current_month && current_day == current_day && current_year == subscription_creation_date.year() {
        return (current_year + 1) as u32;
    }

    return if current_month >= renewal_month && current_day > renewal_day {
        (current_year + 1) as u32
    } else {
        current_year as u32
    };



    //if the original date was the last day in feb, make sure to display the correct date


    2023
}
