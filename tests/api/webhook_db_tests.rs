use claims::{assert_ok};
use newsletter_signup_service::db::webhook_event_db_broker::insert_webhook_event;
use newsletter_signup_service::domain::webhook_event::WebhookEvent;

use crate::helper::spawn_app;

#[tokio::test]
async fn insert_user_works() {
    let app = spawn_app().await;

    let webhook_event = WebhookEvent {
        id: Default::default(),
        event_text: "".to_string(),
        sent_on: Default::default(),
        processed: false,
    };

    let result = insert_webhook_event(&webhook_event, &app.db_pool.clone()).await;
    assert_ok!(result);

}

