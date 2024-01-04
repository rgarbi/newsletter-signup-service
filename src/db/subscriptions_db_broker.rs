use std::str::FromStr;

use chrono::{DateTime, Datelike, NaiveDate, Timelike, Utc};
use sqlx::{PgPool, Postgres, Transaction};
use tracing::log::error;
use uuid::Uuid;

use crate::domain::subscription_models::{
    NewSubscription, OverTheWireSubscription, SubscriptionType,
};
use crate::util::NaiveDateExt;

#[tracing::instrument(
    name = "Saving new subscription details in the database",
    skip(subscription, stripe_subscription_id, transaction)
)]
pub async fn insert_subscription(
    subscription: NewSubscription,
    stripe_subscription_id: String,
    transaction: &mut Transaction<'_, Postgres>,
) -> Result<OverTheWireSubscription, sqlx::Error> {
    let subscription_to_be_saved = OverTheWireSubscription {
        id: Uuid::new_v4(),
        subscriber_id: Uuid::from_str(&subscription.subscriber_id)
            .expect("Unable to parse the UUID"),
        subscription_name: String::from(subscription.subscription_name.as_ref()),
        subscription_mailing_address_line_1: subscription.subscription_mailing_address_line_1,
        subscription_mailing_address_line_2: subscription
            .subscription_mailing_address_line_2
            .unwrap_or_default(),
        subscription_city: subscription.subscription_city,
        subscription_state: subscription.subscription_state,
        subscription_postal_code: subscription.subscription_postal_code,
        subscription_email_address: String::from(subscription.subscription_email_address.as_ref()),
        subscription_creation_date: Utc::now(),
        subscription_cancelled_on_date: None,
        subscription_anniversary_day: subscription.subscription_anniversary_day,
        subscription_anniversary_month: subscription.subscription_anniversary_month,
        subscription_renewal_date: "".to_string(),
        active: true,
        subscription_type: subscription.subscription_type,
        stripe_subscription_id,
    };

    sqlx::query!(
        r#"INSERT INTO subscriptions (
            id, 
            subscriber_id, 
            subscription_name, 
            subscription_mailing_address_line_1, 
            subscription_mailing_address_line_2,
            subscription_city,
            subscription_state,
            subscription_postal_code,
            subscription_email_address,
            subscription_creation_date,
            active,
            subscription_type,
            stripe_subscription_id,
            subscription_cancelled_on_date,
            subscription_anniversary_day,
            subscription_anniversary_month
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16)"#,
        subscription_to_be_saved.id,
        subscription_to_be_saved.subscriber_id,
        subscription_to_be_saved.subscription_name,
        subscription_to_be_saved.subscription_mailing_address_line_1,
        subscription_to_be_saved.subscription_mailing_address_line_2,
        subscription_to_be_saved.subscription_city,
        subscription_to_be_saved.subscription_state,
        subscription_to_be_saved.subscription_postal_code,
        subscription_to_be_saved.subscription_email_address,
        subscription_to_be_saved.subscription_creation_date,
        subscription_to_be_saved.active,
        subscription_to_be_saved.subscription_type.as_str(),
        subscription_to_be_saved.stripe_subscription_id,
        subscription_to_be_saved.subscription_cancelled_on_date,
        subscription_to_be_saved.subscription_anniversary_day as i32,
        subscription_to_be_saved.subscription_anniversary_month as i32
    )
    .execute(&mut **transaction)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;

    Ok(subscription_to_be_saved)
}

#[tracing::instrument(
    name = "Update a subscription by subscription id",
    skip(id, subscription, pool)
)]
pub async fn update_subscription_by_subscription_id(
    id: Uuid,
    subscription: OverTheWireSubscription,
    pool: &PgPool,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"UPDATE subscriptions
            SET
                subscription_name = $1,
                subscription_mailing_address_line_1 = $2,
                subscription_mailing_address_line_2 = $3,
                subscription_city = $4,
                subscription_state = $5,
                subscription_postal_code = $6,
                subscription_email_address = $7
            WHERE id = $8"#,
        subscription.subscription_name,
        subscription.subscription_mailing_address_line_1,
        subscription.subscription_mailing_address_line_2,
        subscription.subscription_city,
        subscription.subscription_state,
        subscription.subscription_postal_code,
        subscription.subscription_email_address,
        id,
    )
    .execute(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;

    Ok(())
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
            subscription_name, 
            subscription_mailing_address_line_1, 
            subscription_mailing_address_line_2,
            subscription_city,
            subscription_state,
            subscription_postal_code,
            subscription_email_address,
            subscription_creation_date,
            subscription_cancelled_on_date,
            subscription_anniversary_day,
            active,
            subscription_type,
            stripe_subscription_id,
            subscription_anniversary_month
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
            subscription_name: row.subscription_name,
            subscription_email_address: row.subscription_email_address,
            subscription_mailing_address_line_1: row.subscription_mailing_address_line_1,
            subscription_mailing_address_line_2: row.subscription_mailing_address_line_2,
            subscription_city: row.subscription_city,
            subscription_state: row.subscription_state,
            subscription_postal_code: row.subscription_postal_code,
            subscription_creation_date: row.subscription_creation_date,
            subscription_cancelled_on_date: row.subscription_cancelled_on_date,
            subscription_anniversary_day: row.subscription_anniversary_day as u32,
            subscription_type: from_str_to_subscription_type(row.subscription_type),
            active: row.active,
            stripe_subscription_id: row.stripe_subscription_id,
            subscription_anniversary_month: row.subscription_anniversary_month as u32,
            subscription_renewal_date: calculate_subscription_renewal_date(
                row.subscription_anniversary_month as u32,
                row.subscription_anniversary_day as u32,
                row.subscription_creation_date,
            )
            .await,
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
            subscription_name,
            subscription_mailing_address_line_1,
            subscription_mailing_address_line_2,
            subscription_city,
            subscription_state,
            subscription_postal_code,
            subscription_email_address,
            subscription_creation_date,
            subscription_cancelled_on_date,
            subscription_anniversary_day,
            active,
            subscription_type,
            stripe_subscription_id,
            subscription_anniversary_month
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
        subscription_name: result.subscription_name,
        subscription_email_address: result.subscription_email_address,
        subscription_mailing_address_line_1: result.subscription_mailing_address_line_1,
        subscription_mailing_address_line_2: result.subscription_mailing_address_line_2,
        subscription_city: result.subscription_city,
        subscription_state: result.subscription_state,
        subscription_postal_code: result.subscription_postal_code,
        subscription_creation_date: result.subscription_creation_date,
        subscription_cancelled_on_date: result.subscription_cancelled_on_date,
        subscription_anniversary_day: result.subscription_anniversary_day as u32,
        subscription_type: from_str_to_subscription_type(result.subscription_type),
        active: result.active,
        stripe_subscription_id: result.stripe_subscription_id,
        subscription_anniversary_month: result.subscription_anniversary_month as u32,
        subscription_renewal_date: calculate_subscription_renewal_date(
            result.subscription_anniversary_month as u32,
            result.subscription_anniversary_day as u32,
            result.subscription_creation_date,
        )
        .await,
    })
}

#[tracing::instrument(
    name = "Cancel a subscription by subscription id",
    skip(id, transaction)
)]
pub async fn cancel_subscription_by_subscription_id(
    id: Uuid,
    transaction: &mut Transaction<'_, Postgres>,
) -> Result<(), sqlx::Error> {
    let cancelled_date = Utc::now();
    sqlx::query!(
        r#"UPDATE subscriptions
            SET active = false,
                subscription_cancelled_on_date = $1
            WHERE id = $2"#,
        cancelled_date,
        id
    )
    .execute(&mut **transaction)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;

    Ok(())
}

#[tracing::instrument(name = "Get all subscriptions", skip(pool))]
pub async fn retrieve_all_subscriptions(
    pool: &PgPool,
) -> Result<Vec<OverTheWireSubscription>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"SELECT
            id, 
            subscriber_id, 
            subscription_name, 
            subscription_mailing_address_line_1, 
            subscription_mailing_address_line_2,
            subscription_city,
            subscription_state,
            subscription_postal_code,
            subscription_email_address,
            subscription_creation_date,
            subscription_cancelled_on_date,
            subscription_anniversary_day,
            active,
            subscription_type,
            stripe_subscription_id,
            subscription_anniversary_month
            FROM subscriptions"#
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
            subscription_name: row.subscription_name,
            subscription_email_address: row.subscription_email_address,
            subscription_mailing_address_line_1: row.subscription_mailing_address_line_1,
            subscription_mailing_address_line_2: row.subscription_mailing_address_line_2,
            subscription_city: row.subscription_city,
            subscription_state: row.subscription_state,
            subscription_postal_code: row.subscription_postal_code,
            subscription_creation_date: row.subscription_creation_date,
            subscription_cancelled_on_date: row.subscription_cancelled_on_date,
            subscription_anniversary_day: row.subscription_anniversary_day as u32,
            subscription_type: from_str_to_subscription_type(row.subscription_type),
            active: row.active,
            stripe_subscription_id: row.stripe_subscription_id,
            subscription_anniversary_month: row.subscription_anniversary_month as u32,
            subscription_renewal_date: calculate_subscription_renewal_date(
                row.subscription_anniversary_month as u32,
                row.subscription_anniversary_day as u32,
                row.subscription_creation_date,
            )
            .await,
        })
    }
    Ok(subscriptions)
}

pub fn from_str_to_subscription_type(val: String) -> SubscriptionType {
    if val.eq("Digital") {
        return SubscriptionType::Digital;
    }

    if val.eq("Paper") {
        return SubscriptionType::Paper;
    }

    error!("Could not map string: {} to the enum SubscriptionType", val);
    SubscriptionType::Digital
}

async fn calculate_subscription_renewal_date(
    renewal_month: u32,
    renewal_day: u32,
    subscription_creation_date: DateTime<Utc>,
) -> String {
    let mut local_copy_renewal_day = renewal_day;
    let mut local_copy_subscription_creation_date = subscription_creation_date;
    let renewal_year = calculate_renewal_year(
        renewal_month,
        local_copy_renewal_day,
        local_copy_subscription_creation_date,
    )
    .await;

    let is_renewal_year_a_leap_year =
        NaiveDate::parse_from_str(format!("{}-01-01", renewal_year).as_str(), "%Y-%m-%d")
            .unwrap()
            .is_leap_year();

    if renewal_month == 2 && local_copy_renewal_day == 29 && !is_renewal_year_a_leap_year {
        local_copy_renewal_day = 28;
        local_copy_subscription_creation_date = DateTime::from_utc(
            NaiveDate::parse_from_str(
                format!(
                    "{}-{}-{}",
                    renewal_year, renewal_month, local_copy_renewal_day
                )
                .as_str(),
                "%Y-%m-%d",
            )
            .unwrap()
            .and_hms_opt(
                local_copy_subscription_creation_date.hour(),
                local_copy_subscription_creation_date.minute(),
                local_copy_subscription_creation_date.second(),
            )
            .unwrap(),
            Utc,
        );
    }

    let renewal_date = local_copy_subscription_creation_date
        .with_year(renewal_year as i32)
        .unwrap()
        .date_naive();

    format!(
        "{}/{}/{}",
        renewal_date.month(),
        renewal_date.day(),
        renewal_date.year()
    )
}

async fn calculate_renewal_year(
    renewal_month: u32,
    renewal_day: u32,
    subscription_creation_date: DateTime<Utc>,
) -> u32 {
    let current_day = Utc::now().day();
    let current_month = Utc::now().month();
    let current_year = Utc::now().year();

    if current_month == renewal_month
        && current_day == renewal_day
        && current_year == subscription_creation_date.year()
    {
        return (current_year + 1) as u32;
    }

    if current_month == renewal_month && current_day == renewal_day {
        return current_year as u32;
    }

    if current_month == renewal_month && current_day > renewal_day {
        return (current_year + 1) as u32;
    }

    if current_month == renewal_month && current_day < renewal_day {
        return current_year as u32;
    }

    if current_month > renewal_month {
        return (current_year + 1) as u32;
    }

    current_year as u32
}

#[cfg(test)]
mod tests {
    use crate::db::subscriptions_db_broker::calculate_renewal_year;
    use crate::util::NaiveDateExt;
    use chrono::{DateTime, Datelike, NaiveDate, Utc};

    #[tokio::test]
    async fn calculate_renewal_year_test() {
        let now = Utc::now();
        let subscription_year_current_year: u32 = now.year() as u32;

        let date_string_january = format!("{}-01-01", subscription_year_current_year);
        let subscription_creation_date = DateTime::from_utc(
            NaiveDate::parse_from_str(date_string_january.as_str(), "%Y-%m-%d")
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap(),
            Utc,
        );
        assert_eq!(
            calculate_renewal_year(1, 1, subscription_creation_date).await,
            subscription_year_current_year + 1
        );

        let date_string_february = format!("{}-02-01", subscription_year_current_year);
        let subscription_creation_date = DateTime::from_utc(
            NaiveDate::parse_from_str(date_string_february.as_str(), "%Y-%m-%d")
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap(),
            Utc,
        );
        assert_eq!(
            calculate_renewal_year(2, 1, subscription_creation_date).await,
            subscription_year_current_year + 1
        );

        let day: u32 = NaiveDate::parse_from_str(
            format!("{}-02-01", subscription_year_current_year).as_str(),
            "%Y-%m-%d",
        )
        .unwrap()
        .days_in_month() as u32;
        let date_string_february_end_of_month =
            format!("{}-02-{}", subscription_year_current_year, day);
        NaiveDate::parse_from_str(date_string_february_end_of_month.as_str(), "%Y-%m-%d")
            .unwrap()
            .days_in_month();
        let subscription_creation_date = DateTime::from_utc(
            NaiveDate::parse_from_str(date_string_february_end_of_month.as_str(), "%Y-%m-%d")
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap(),
            Utc,
        );
        assert_eq!(
            calculate_renewal_year(2, day, subscription_creation_date).await,
            subscription_year_current_year + 1
        );
    }
}
