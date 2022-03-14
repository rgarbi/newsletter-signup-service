use std::str::FromStr;

use actix_web::{web, HttpResponse, Responder};
use chrono::Utc;
use sqlx::PgPool;
use tracing::log::error;
use uuid::Uuid;

use crate::domain::new_subscription::{
    NewSubscription, OverTheWireCreateSubscription, OverTheWireSubscription, SubscriptionType,
};
use crate::domain::valid_email::ValidEmail;
use crate::domain::valid_name::ValidName;
use crate::util::from_string_to_uuid;

impl TryFrom<OverTheWireCreateSubscription> for NewSubscription {
    type Error = String;
    fn try_from(subscription: OverTheWireCreateSubscription) -> Result<Self, Self::Error> {
        let subscription_first_name = ValidName::parse(subscription.subscription_first_name)?;
        let subscription_last_name = ValidName::parse(subscription.subscription_last_name)?;
        let subscription_email_address =
            ValidEmail::parse(subscription.subscription_email_address)?;
        Ok(NewSubscription {
            subscriber_id: subscription.subscriber_id,
            subscription_first_name,
            subscription_last_name,
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
    match insert_subscription(new_subscription, &pool).await {
        Ok(subscription) => HttpResponse::Ok().json(subscription),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[tracing::instrument(
    name = "Getting subscriptions by subscriber id",
    skip(id, pool),
    fields(
        id = %id,
    )
)]
pub async fn get_subscriptions_by_subscriber_id(
    id: web::Path<String>,
    pool: web::Data<PgPool>,
) -> impl Responder {
    match retrieve_subscriptions_by_subscriber_id(from_string_to_uuid(id).unwrap(), &pool).await {
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
    match retrieve_subscription_by_subscription_id(from_string_to_uuid(id).unwrap(), &pool).await {
        Ok(subscriptions) => HttpResponse::Ok().json(subscriptions),
        Err(_) => HttpResponse::NotFound().finish(),
    }
}

#[tracing::instrument(
    name = "Saving new subscription details in the database",
    skip(subscription, pool)
)]
pub async fn insert_subscription(
    subscription: NewSubscription,
    pool: &PgPool,
) -> Result<OverTheWireSubscription, sqlx::Error> {
    let subscription_to_be_saved = OverTheWireSubscription {
        id: Uuid::new_v4(),
        subscriber_id: Uuid::from_str(&*subscription.subscriber_id)
            .expect("Unable to parse the UUID"),
        subscription_first_name: String::from(subscription.subscription_first_name.as_ref()),
        subscription_last_name: String::from(subscription.subscription_last_name.as_ref()),
        subscription_mailing_address_line_1: subscription.subscription_mailing_address_line_1,
        subscription_mailing_address_line_2: subscription
            .subscription_mailing_address_line_2
            .unwrap_or_default(),
        subscription_city: subscription.subscription_city,
        subscription_state: subscription.subscription_state,
        subscription_postal_code: subscription.subscription_postal_code,
        subscription_email_address: String::from(subscription.subscription_email_address.as_ref()),
        subscription_creation_date: Utc::now(),
        active: true,
        subscription_type: subscription.subscription_type,
    };

    sqlx::query!(
        r#"INSERT INTO subscriptions (
            id, 
            subscriber_id, 
            subscription_first_name, 
            subscription_last_name, 
            subscription_mailing_address_line_1, 
            subscription_mailing_address_line_2,
            subscription_city,
            subscription_state,
            subscription_postal_code,
            subscription_email_address,
            subscription_creation_date,
            active,
            subscription_type
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)"#,
        subscription_to_be_saved.id,
        subscription_to_be_saved.subscriber_id,
        subscription_to_be_saved.subscription_first_name,
        subscription_to_be_saved.subscription_last_name,
        subscription_to_be_saved.subscription_mailing_address_line_1,
        subscription_to_be_saved.subscription_mailing_address_line_2,
        subscription_to_be_saved.subscription_city,
        subscription_to_be_saved.subscription_state,
        subscription_to_be_saved.subscription_postal_code,
        subscription_to_be_saved.subscription_email_address,
        subscription_to_be_saved.subscription_creation_date,
        subscription_to_be_saved.active,
        subscription_to_be_saved.subscription_type.as_str()
    )
    .execute(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;

    Ok(subscription_to_be_saved)
}

#[tracing::instrument(name = "Get all subscriptions by subscriber id", skip(id, pool))]
pub async fn retrieve_subscriptions_by_subscriber_id(
    id: Uuid,
    pool: &PgPool,
) -> Result<Vec<OverTheWireSubscription>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"SELECT
            id, 
            subscriber_id, 
            subscription_first_name, 
            subscription_last_name, 
            subscription_mailing_address_line_1, 
            subscription_mailing_address_line_2,
            subscription_city,
            subscription_state,
            subscription_postal_code,
            subscription_email_address,
            subscription_creation_date,
            active,
            subscription_type
            FROM subscriptions WHERE subscriber_id = $1"#,
        id
    )
    .fetch_all(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;

    let mut subscriptions: Vec<OverTheWireSubscription> = Vec::new();

    for row in rows {
        subscriptions.push(OverTheWireSubscription {
            id: row.id,
            subscriber_id: row.subscriber_id,
            subscription_first_name: row.subscription_first_name,
            subscription_last_name: row.subscription_last_name,
            subscription_email_address: row.subscription_email_address,
            subscription_mailing_address_line_1: row.subscription_mailing_address_line_1,
            subscription_mailing_address_line_2: row.subscription_mailing_address_line_2,
            subscription_city: row.subscription_city,
            subscription_state: row.subscription_state,
            subscription_postal_code: row.subscription_postal_code,
            subscription_creation_date: row.subscription_creation_date,
            subscription_type: from_str_to_subscription_type(row.subscription_type),
            active: row.active,
        })
    }
    Ok(subscriptions)
}

#[tracing::instrument(name = "Get subscription by subscription id", skip(id, pool))]
pub async fn retrieve_subscription_by_subscription_id(
    id: Uuid,
    pool: &PgPool,
) -> Result<OverTheWireSubscription, sqlx::Error> {
    let result = sqlx::query!(
        r#"SELECT
            id,
            subscriber_id,
            subscription_first_name,
            subscription_last_name,
            subscription_mailing_address_line_1,
            subscription_mailing_address_line_2,
            subscription_city,
            subscription_state,
            subscription_postal_code,
            subscription_email_address,
            subscription_creation_date,
            active,
            subscription_type
            FROM subscriptions WHERE id = $1"#,
        id
    )
    .fetch_one(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;

    Ok(OverTheWireSubscription {
        id: result.id,
        subscriber_id: result.subscriber_id,
        subscription_first_name: result.subscription_first_name,
        subscription_last_name: result.subscription_last_name,
        subscription_email_address: result.subscription_email_address,
        subscription_mailing_address_line_1: result.subscription_mailing_address_line_1,
        subscription_mailing_address_line_2: result.subscription_mailing_address_line_2,
        subscription_city: result.subscription_city,
        subscription_state: result.subscription_state,
        subscription_postal_code: result.subscription_postal_code,
        subscription_creation_date: result.subscription_creation_date,
        subscription_type: from_str_to_subscription_type(result.subscription_type),
        active: result.active,
    })
}

pub fn from_str_to_subscription_type(val: String) -> SubscriptionType {
    if val.eq("Electronic") {
        return SubscriptionType::Electronic;
    }

    if val.eq("Physical") {
        return SubscriptionType::Physical;
    }

    error!("Could not map string: {} to the enum SubscriptionType", val);
    SubscriptionType::Electronic
}
