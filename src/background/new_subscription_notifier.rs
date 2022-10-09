use crate::configuration::get_configuration;
use crate::db::subscriptions_db_broker::retrieve_subscription_by_subscription_id;
use crate::domain::subscription_models::OverTheWireSubscription;
use crate::domain::valid_email::ValidEmail;
use crate::email_client::EmailClient;
use sqlx::PgPool;
use uuid::Uuid;

pub fn notify_of_new_subscription(subscription_id: Uuid, email_client: EmailClient, pool: &PgPool) {
    let new_pool = pool.clone();

    tokio::spawn(async move {
        notify_subscriber(subscription_id, &email_client, &new_pool).await;
    });
}

pub async fn notify_subscriber(subscription_id: Uuid, email_client: &EmailClient, pool: &PgPool) {
    if let Ok(subscription) = retrieve_subscription_by_subscription_id(subscription_id, pool).await
    {
        let recipients = get_configuration()
            .unwrap()
            .application_feature_settings
            .subscription_notification_addresses
            .iter()
            .map(|recipient| ValidEmail::parse(recipient.clone()).unwrap())
            .collect::<Vec<ValidEmail>>();

        let _ = email_client
            .send_email(
                recipients,
                "New Subscription",
                html_content(&subscription).as_str(),
                text_content(&subscription).as_str(),
            )
            .await
            .is_ok();
    }
}

fn text_content(subscription: &OverTheWireSubscription) -> String {
    format!(
        r#"A new subscription was created. The details are below:
        Subscription Type: {}
        Subscription Name: {}
        Subscription Address Line 1: {}
        Subscription Address Line 2: {}
        Subscription City: {}
        Subscription State: {}
        Subscription Postal Code: {}
        Subscription Date: {}
        Subscription Email: {}"#,
        subscription.subscription_type.as_str(),
        subscription.subscription_name,
        subscription.subscription_mailing_address_line_1,
        subscription.subscription_mailing_address_line_2,
        subscription.subscription_city,
        subscription.subscription_state,
        subscription.subscription_postal_code,
        subscription.subscription_creation_date,
        subscription.subscription_email_address,
    )
}

fn html_content(subscription: &OverTheWireSubscription) -> String {
    format!(
        r#"<html>
                <h3>A new subscription was created.</h3>
                <p>The details are below:</p>
                <table>
                    <tr>
                        <td>Subscription Type: {}</td>
                    </tr>
                    <tr>
                        <td>Subscription Name: {}</td>
                    </tr>
                    <tr>
                        <td>Subscription Address Line 1: {}</td>
                    </tr>
                    <tr>
                        <td>Subscription Address Line 2: {}</td>
                    </tr>
                    <tr>
                        <td>Subscription City: {}</td>
                    </tr>
                    <tr>
                        <td>Subscription State: {}</td>
                    </tr>
                    <tr>
                        <td>Subscription Postal Code: {}</td>
                    </tr>
                    <tr>
                        <td>Subscription Date: {}</td>
                    </tr>
                    <tr>
                        <td>Subscription Email: {}</td>
                    </tr>
                </table>"#,
        subscription.subscription_type.as_str(),
        subscription.subscription_name,
        subscription.subscription_mailing_address_line_1,
        subscription.subscription_mailing_address_line_2,
        subscription.subscription_city,
        subscription.subscription_state,
        subscription.subscription_postal_code,
        subscription.subscription_creation_date,
        subscription.subscription_email_address
    )
}

#[cfg(test)]
mod tests {
    use crate::background::new_subscription_notifier::{html_content, text_content};
    use crate::domain::subscription_models::{OverTheWireSubscription, SubscriptionType};
    use chrono::{Datelike, Utc};
    use uuid::Uuid;

    fn get_sub() -> OverTheWireSubscription {
        OverTheWireSubscription {
            id: Uuid::new_v4(),
            subscriber_id: Uuid::new_v4(),
            subscription_name: "Joe Smith".to_string(),
            subscription_mailing_address_line_1: "123 Main".to_string(),
            subscription_mailing_address_line_2: "n/a".to_string(),
            subscription_city: "Kansas City".to_string(),
            subscription_state: "MO".to_string(),
            subscription_postal_code: "64105".to_string(),
            subscription_email_address: "someone@gmail.com".to_string(),
            subscription_creation_date: Utc::now(),
            subscription_cancelled_on_date: None,
            subscription_anniversary_day: Utc::now().day(),
            subscription_anniversary_month: Utc::now().month(),
            active: true,
            subscription_type: SubscriptionType::Digital,
            stripe_subscription_id: Uuid::new_v4().to_string(),
        }
    }

    #[test]
    fn text_content_works() {
        let subscription = get_sub();

        let email_text_content = text_content(&subscription);
        assert_eq!(
            true,
            email_text_content.contains(&subscription.subscription_name.as_str())
        )
    }

    #[test]
    fn html_content_works() {
        let subscription = get_sub();

        let email_html_content = html_content(&subscription);
        assert_eq!(
            true,
            email_html_content.contains(&subscription.subscription_name.as_str())
        )
    }
}
