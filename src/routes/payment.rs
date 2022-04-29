use actix_web::{web, HttpRequest, HttpResponse, Responder};
use secrecy::ExposeSecret;
use serde::de::Unexpected::Option;
use serde_json::json;
use sqlx::PgPool;
use std::str::FromStr;
use stripe::{CheckoutSessionMode, CustomerId, Webhook, WebhookEvent};

use crate::auth::token::Claims;
use crate::configuration::get_configuration;
use crate::db::checkout_session_db_broker::{
    insert_checkout_session, retrieve_checkout_session_by_stripe_session_id,
    set_checkout_session_state_to_success_by_stripe_session_id,
};
use crate::db::subscribers_db_broker::retrieve_subscriber_by_id;
use crate::db::subscriptions_db_broker::insert_subscription;
use crate::domain::checkout_models::{CreateCheckoutSession, CreateCheckoutSessionRedirect};
use crate::domain::subscriber_models::OverTheWireSubscriber;
use crate::domain::subscription_models::{NewSubscription, OverTheWireCreateSubscription};
use crate::util::from_string_to_uuid;

#[tracing::instrument(
    name = "Create checkout session",
    skip(create_checkout_session, user_id, pool, user),
    fields(
        price_param = %create_checkout_session.price_lookup_key,
    )
)]
pub async fn create_checkout_session(
    create_checkout_session: web::Json<CreateCheckoutSession>,
    user_id: web::Path<String>,
    pool: web::Data<PgPool>,
    user: Claims,
) -> impl Responder {
    if user_id.clone() != user.user_id {
        return HttpResponse::Unauthorized().finish();
    }

    let new_subscription: NewSubscription =
        match create_checkout_session.subscription.clone().try_into() {
            Ok(subscription) => subscription,
            Err(_) => return HttpResponse::BadRequest().finish(),
        };

    //get the subscriber by id
    let subscriber = match retrieve_subscriber_by_id(
        from_string_to_uuid(new_subscription.subscriber_id.as_str()).unwrap(),
        &pool,
    )
    .await
    {
        Ok(subscriber) => subscriber,
        Err(err) => {
            println!("Err: {:?}", err);
            return HttpResponse::BadRequest().finish();
        }
    };

    let configuration = get_configuration().unwrap();

    let success_url: String = format!(
        "{}/checkout-success?session_id={{CHECKOUT_SESSION_ID}}",
        &configuration.application.web_app_host
    );
    let cancel_url: String = format!(
        "{}/checkout-cancel",
        &configuration.application.web_app_host
    );

    let look_up_keys = [create_checkout_session.price_lookup_key.clone()].to_vec();
    let client = stripe::Client::new(configuration.stripe_client.api_secret_key.expose_secret());
    let mut list_prices = stripe::ListPrices::new();
    list_prices.lookup_keys = Some(Box::new(look_up_keys));
    let list_prices_response = stripe::Price::list(&client, list_prices).await;

    match list_prices_response {
        Ok(prices) => {
            println!("Got prices: {:?}", &prices);
            let price_id = prices.data[0].id.to_string();

            let line_item = stripe::CreateCheckoutSessionLineItems {
                adjustable_quantity: None,
                description: None,
                dynamic_tax_rates: None,
                price: Some(Box::new(price_id.to_string())),
                price_data: None,
                quantity: Some(Box::new(1)),
                tax_rates: None,
            };
            let line_items = [line_item].to_vec();

            let mut checkout_session =
                stripe::CreateCheckoutSession::new(cancel_url.as_str(), success_url.as_str());
            checkout_session.line_items = Some(Box::new(line_items));
            checkout_session.mode = Some(CheckoutSessionMode::Subscription);
            checkout_session.customer = set_stripe_customer_id(subscriber);

            let checkout_session_response =
                stripe::CheckoutSession::create(&client, checkout_session).await;

            match checkout_session_response {
                Ok(checkout_session_created) => {
                    let checkout_session_url = checkout_session_created.url.clone().unwrap();
                    println!(
                        "Checkout session Created!!! URL: {:?}",
                        &checkout_session_created.url
                    );
                    println!(
                        "Checkout session Created!!! ID: {:?}",
                        &checkout_session_created.id
                    );

                    println!(
                        "CUSTOMER ID: {}",
                        &checkout_session_created.customer.unwrap().id()
                    );

                    let store_checkout_result = insert_checkout_session(
                        user_id.into_inner().clone(),
                        create_checkout_session.price_lookup_key.clone(),
                        new_subscription,
                        checkout_session_created.id.as_str().to_string(),
                        &pool,
                    )
                    .await;

                    match store_checkout_result {
                        Ok(_) => {
                            println!("REDIRECTING!!!");
                            let redirect_response = CreateCheckoutSessionRedirect {
                                location: checkout_session_url.as_str().to_string(),
                            };
                            HttpResponse::Ok().json(redirect_response)
                        }
                        Err(err) => {
                            println!("Err: {:?}", err);
                            HttpResponse::InternalServerError().finish()
                        }
                    }
                }
                Err(stripe_error) => {
                    println!("Err: {:?}", stripe_error);
                    HttpResponse::InternalServerError().finish()
                }
            }
        }
        Err(err) => {
            println!("Err: {:?}", err);
            HttpResponse::InternalServerError().finish()
        }
    }
}

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

#[tracing::instrument(
    name = "Complete Session",
    skip(params, pool, user),
    fields(
        user_id = %params.0,
        session_id = %params.1,
    )
)]
pub async fn complete_session(
    params: web::Path<(String, String)>,
    pool: web::Data<PgPool>,
    user: Claims,
) -> impl Responder {
    let param_tuple: (String, String) = params.into_inner();
    let user_id = param_tuple.clone().0;
    let session_id = param_tuple.clone().1;
    if user_id != user.user_id {
        return HttpResponse::Unauthorized().finish();
    }
    let checkout_session = retrieve_checkout_session_by_stripe_session_id(&session_id, &pool).await;

    return match checkout_session {
        Ok(checkout) => {
            if checkout.user_id != user_id {
                return HttpResponse::Unauthorized().finish();
            }

            let mut transaction = match pool.begin().await {
                Ok(transaction) => transaction,
                Err(_) => return HttpResponse::InternalServerError().finish(),
            };
            //use a transaction
            let set_state_result = set_checkout_session_state_to_success_by_stripe_session_id(
                &checkout.stripe_session_id,
                &mut transaction,
            )
            .await;

            if set_state_result.is_err() {
                transaction.rollback().await.unwrap();
                return HttpResponse::InternalServerError().finish();
            }

            let stored_subscription: OverTheWireCreateSubscription =
                serde_json::from_value(checkout.subscription).unwrap();

            let new_subscription = match stored_subscription.try_into() {
                Ok(subscription) => subscription,
                Err(_) => return HttpResponse::BadRequest().finish(),
            };

            let subscription_result = insert_subscription(
                new_subscription,
                checkout.stripe_session_id,
                &mut transaction,
            )
            .await;

            match subscription_result {
                Ok(_) => {
                    if transaction.commit().await.is_err() {
                        HttpResponse::InternalServerError().finish();
                    }
                    HttpResponse::Ok().json(json!({}))
                }
                Err(_) => {
                    transaction.rollback().await.unwrap();
                    HttpResponse::InternalServerError().finish()
                }
            }
        }
        Err(err) => {
            println!("Err: {:?}", err);
            HttpResponse::NotFound().finish()
        }
    };
}

fn set_stripe_customer_id(subscriber: &OverTheWireSubscriber) -> Option<CustomerId> {
    if subscriber.stripe_customer_id.is_some() {
        return Some(
            CustomerId::from_str(subscriber.stripe_customer_id.unwrap().as_str()).unwrap(),
        );
    } else {
        None
    }
}
