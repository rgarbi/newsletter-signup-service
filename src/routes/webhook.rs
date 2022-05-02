use actix_web::{web, HttpRequest, HttpResponse, Responder};
use core::option::Option;
use secrecy::ExposeSecret;
use serde_json::json;
use sqlx::PgPool;
use std::str::FromStr;
use stripe::{
    CheckoutSessionMode, Client, CreateCustomer, Customer, CustomerId, StripeError, Webhook,
    WebhookEvent,
};

use crate::auth::token::Claims;
use crate::configuration::get_configuration;
use crate::db::checkout_session_db_broker::{
    insert_checkout_session, retrieve_checkout_session_by_stripe_session_id,
    set_checkout_session_state_to_success_by_stripe_session_id,
};
use crate::db::subscribers_db_broker::{retrieve_subscriber_by_id, set_stripe_customer_id};
use crate::db::subscriptions_db_broker::insert_subscription;
use crate::domain::checkout_models::{CreateCheckoutSession, CreateCheckoutSessionRedirect};
use crate::domain::subscriber_models::OverTheWireSubscriber;
use crate::domain::subscription_models::{NewSubscription, OverTheWireCreateSubscription};
use crate::util::from_string_to_uuid;

#[tracing::instrument(
name = "Handle Webhook",
    skip(webhook_event, _pool, _user),
    fields(
        webhook_event_id = %webhook_event.id,
    )
)]
pub async fn handle_webhook(
    webhook_event: web::Json<WebhookEvent>,
    req: HttpRequest,
    body: web::Bytes,
    _pool: web::Data<PgPool>,
    _user: Claims,
) -> impl Responder {
    let configuration = get_configuration().unwrap();
    println!("Got a webhook event the ID was: {}", webhook_event.id);
    let stripe_signature_header = req.headers().get("Stripe-Signature");

    if let Some(..) = stripe_signature_header {
        let signature = stripe_signature_header.unwrap().to_str().ok().unwrap();
        let body = std::str::from_utf8(&body).unwrap();
        println!("Got a webhook event the hash was: {}", &signature);

        let validate_signature = Webhook::construct_event(
            body,
            signature,
            configuration
                .stripe_client
                .webhook_key
                .expose_secret()
                .as_str(),
        );

        match validate_signature {
            Ok(_webhook_event) => {
                println!("Successfully validated the webhook!!!");
                println!("Web hook type was: {:?}", _webhook_event.event_type);
                println!("Web hook object was: {:?}", _webhook_event.data.object);
            }
            Err(webhook_error) => {
                println!("Err: {:?}", webhook_error);
            }
        }
    }

    HttpResponse::Ok().json(json!({}))
}
