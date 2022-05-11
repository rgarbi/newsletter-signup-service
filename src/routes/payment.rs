use actix_web::{web, HttpResponse, Responder};
use reqwest::Error;
use secrecy::ExposeSecret;
use serde_json::json;
use sqlx::PgPool;
use std::str::FromStr;
use stripe::{CheckoutSessionMode, Client, CreateCustomer, Customer, CustomerId, StripeError};

use crate::auth::token::Claims;
use crate::configuration::get_configuration;
use crate::db::checkout_session_db_broker::{
    insert_checkout_session, retrieve_checkout_session_by_stripe_session_id,
    set_checkout_session_state_to_success_by_stripe_session_id,
};
use crate::db::subscribers_db_broker::{
    retrieve_subscriber_by_id, retrieve_subscriber_by_user_id, set_stripe_customer_id,
};
use crate::db::subscriptions_db_broker::insert_subscription;
use crate::domain::checkout_models::{
    CreateCheckoutSession, CreateStripeSessionRedirect, StripeBillingPortalSession,
    StripeSessionObject,
};
use crate::domain::subscriber_models::OverTheWireSubscriber;
use crate::domain::subscription_models::{NewSubscription, OverTheWireCreateSubscription};
use crate::stripe_client::StripeClient;
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

    //Get customer
    let stripe_customer_id = match get_stripe_customer_id(&subscriber, &client).await {
        Ok(id) => id,
        Err(error_response) => {
            return error_response;
        }
    };

    set_stripe_customer_id(&subscriber.id, stripe_customer_id.as_str(), &pool)
        .await
        .unwrap();

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
            let client_ref_id = subscriber.id.to_string().clone();
            checkout_session.client_reference_id = Option::Some(client_ref_id.as_str());

            checkout_session.customer =
                Option::Some(CustomerId::from_str(stripe_customer_id.as_str()).unwrap());

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
                            let redirect_response = CreateStripeSessionRedirect {
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
    let config = get_configuration().unwrap();
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

            //Get the actual subscription id
            let stripe_session = match get_stripe_session(
                checkout.stripe_session_id.clone(),
                config
                    .stripe_client
                    .api_secret_key
                    .expose_secret()
                    .to_string(),
            )
            .await
            {
                Ok(session) => session,
                Err(err) => {
                    println!(
                        "Something blew up when getting the stripe session! {:?}",
                        err
                    );
                    return HttpResponse::InternalServerError().finish();
                }
            };

            let subscription_result = insert_subscription(
                new_subscription,
                stripe_session.subscription.unwrap(),
                &mut transaction,
            )
            .await;

            match subscription_result {
                Ok(_) => {
                    if transaction.commit().await.is_err() {
                        HttpResponse::InternalServerError().finish()
                    } else {
                        HttpResponse::Ok().json(json!({}))
                    }
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

#[tracing::instrument(
    name = "Create Stripe Portal Session",
    skip(user_id, pool, user, stripe_client),
    fields(
        user_id = %user_id,
    )
)]
pub async fn create_stripe_portal_session(
    user_id: web::Path<String>,
    pool: web::Data<PgPool>,
    user: Claims,
    stripe_client: web::Data<StripeClient>,
) -> impl Responder {
    let config = get_configuration().unwrap();
    if user_id.clone() != user.user_id {
        return HttpResponse::Unauthorized().finish();
    }

    let subscriber =
        match retrieve_subscriber_by_user_id(user_id.into_inner().as_str(), &pool).await {
            Ok(subscriber) => subscriber,
            Err(err) => {
                println!("Err: {:?}", err);
                return HttpResponse::BadRequest().finish();
            }
        };

    if subscriber.stripe_customer_id.is_none() {
        return HttpResponse::BadRequest().finish();
    }

    let return_url = format!("{}/subscriber", config.application.web_app_host);

    let result = stripe_client
        .create_billing_portal_session(subscriber.stripe_customer_id.unwrap(), return_url)
        .await;

    return match result {
        Ok(stripe_billing_portal_session) => {
            println!(
                "Got the following back {:?}",
                &stripe_billing_portal_session.url
            );
            let redirect_response = CreateStripeSessionRedirect {
                location: stripe_billing_portal_session.url,
            };
            HttpResponse::Ok().json(redirect_response)
        }
        Err(err) => {
            println!(
                "Something blew up when creating the portal session! {:?}",
                err
            );
            HttpResponse::InternalServerError().finish()
        }
    };
}

async fn get_stripe_customer_id(
    subscriber: &OverTheWireSubscriber,
    client: &Client,
) -> Result<String, HttpResponse> {
    if subscriber.stripe_customer_id.is_none() {
        return match create_stripe_customer(subscriber.email_address.clone(), client).await {
            Ok(customer) => Ok(String::from(customer.id.as_str())),
            Err(err) => {
                println!("Err: {:?}", err);
                Err(HttpResponse::InternalServerError().finish())
            }
        };
    } else {
        Ok(subscriber.stripe_customer_id.clone().unwrap())
    }
}

async fn create_stripe_customer(email: String, client: &Client) -> Result<Customer, StripeError> {
    let mut create_customer_params = CreateCustomer::new();
    create_customer_params.email = Option::Some(email.as_str());
    stripe::Customer::create(client, create_customer_params).await
}

async fn get_stripe_session(
    stripe_session_id: String,
    stripe_publishable_key: String,
) -> Result<StripeSessionObject, Error> {
    let response = reqwest::Client::new()
        .get(format!(
            "https://api.stripe.com/v1/checkout/sessions/{}",
            stripe_session_id
        ))
        .basic_auth(stripe_publishable_key, Option::Some(String::new()))
        .send()
        .await;

    return match response {
        Ok(response) => {
            let response_body = response.text().await.unwrap();
            println!("Got the following back!! {:?}", &response_body);
            let stripe_session: StripeSessionObject =
                serde_json::from_str(response_body.as_str()).unwrap();
            Ok(stripe_session)
        }
        Err(err) => {
            println!("Err: {:?}", err);
            Err(err)
        }
    };
}
