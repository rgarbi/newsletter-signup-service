use crate::auth::token::Claims;
use crate::db::subscribers::retrieve_subscriber_by_id;
use actix_web::{web, HttpResponse, Responder};
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

use crate::db::subscriptions_db_broker::{
    insert_subscription, retrieve_subscription_by_subscription_id,
    retrieve_subscriptions_by_subscriber_id,
};
use crate::domain::subscription_models::{NewSubscription, OverTheWireCreateSubscription};
use crate::domain::valid_email::ValidEmail;
use crate::domain::valid_name::ValidName;
use crate::util::from_string_to_uuid;

impl TryFrom<OverTheWireCreateSubscription> for NewSubscription {
    type Error = String;
    fn try_from(subscription: OverTheWireCreateSubscription) -> Result<Self, Self::Error> {
        let subscription_name = ValidName::parse(subscription.subscription_name)?;
        let subscription_email_address =
            ValidEmail::parse(subscription.subscription_email_address)?;
        Ok(NewSubscription {
            subscriber_id: subscription.subscriber_id,
            subscription_name,
            subscription_email_address,
            subscription_mailing_address_line_1: subscription.subscription_mailing_address_line_1,
            subscription_mailing_address_line_2: subscription.subscription_mailing_address_line_2,
            subscription_city: subscription.subscription_city,
            subscription_state: subscription.subscription_state,
            subscription_postal_code: subscription.subscription_postal_code,
            subscription_type: subscription.subscription_type,
            subscription_creation_date: Utc::now(),
            active: true,
        })
    }
}

#[tracing::instrument(
    name = "Adding a new subscriber",
    skip(subscription, pool),
    fields(
        subscriber_id = %subscription.subscriber_id,
        subscription_email_address = %subscription.subscription_email_address,
    )
)]
pub async fn post_subscription(
    subscription: web::Json<OverTheWireCreateSubscription>,
    pool: web::Data<PgPool>,
) -> impl Responder {
    let new_subscription = match subscription.0.try_into() {
        Ok(subscription) => subscription,
        Err(_) => return HttpResponse::BadRequest().finish(),
    };
    match insert_subscription(new_subscription, Uuid::new_v4().to_string(), &pool).await {
        Ok(subscription) => HttpResponse::Ok().json(subscription),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

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
    match retrieve_subscriber_by_id(from_string_to_uuid(&id).unwrap(), &pool).await {
        Ok(subscriber) => {
            if subscriber.user_id != user.user_id {
                return HttpResponse::Unauthorized().finish();
            }
        }
        Err(_) => return HttpResponse::BadRequest().finish(),
    }

    match retrieve_subscriptions_by_subscriber_id(from_string_to_uuid(&id).unwrap(), &pool).await {
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
    match retrieve_subscription_by_subscription_id(from_string_to_uuid(&id).unwrap(), &pool).await {
        Ok(subscriptions) => HttpResponse::Ok().json(subscriptions),
        Err(_) => HttpResponse::NotFound().finish(),
    }
}
