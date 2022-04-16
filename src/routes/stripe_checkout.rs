use actix_web::{web, HttpResponse, Responder};
use secrecy::ExposeSecret;
use serde_json::json;
use sqlx::PgPool;
use stripe::CheckoutSessionMode;

use crate::auth::token::Claims;
use crate::configuration::get_configuration;
use crate::domain::stripe_models::CreateCheckoutSession;

#[tracing::instrument(
    name = "Create checkout session",
    skip(create_checkout_session, _pool, _user),
    fields(
        price_param = %create_checkout_session.price_lookup_key,
    )
)]
pub async fn create_checkout_session(
    create_checkout_session: web::Json<CreateCheckoutSession>,
    _pool: web::Data<PgPool>,
    _user: Claims,
) -> impl Responder {
    let configuration = get_configuration().unwrap();

    let success_url: String = format!(
        "{}/checkout-success",
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
            let checkout_session_response =
                stripe::CheckoutSession::create(&client, checkout_session).await;

            match checkout_session_response {
                Ok(checkout_session_created) => {
                    println!(
                        "Checkout session Created!!! URL: {:?}",
                        checkout_session_created.url
                    );
                    HttpResponse::Ok().json(json!({}))
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
