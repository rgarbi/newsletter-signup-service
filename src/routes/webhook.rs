use actix_web::{web, HttpRequest, HttpResponse, Responder};
use secrecy::ExposeSecret;
use serde_json::json;
use sqlx::PgPool;

use stripe::Webhook;

use crate::configuration::get_configuration;

#[tracing::instrument(name = "Handle Webhook", skip(req, body, _pool), fields())]
pub async fn handle_webhook(
    req: HttpRequest,
    body: web::Bytes,
    _pool: web::Data<PgPool>,
) -> impl Responder {
    let configuration = get_configuration().unwrap();
    let stripe_signature_header = req.headers().get("Stripe-Signature");

    if let Some(..) = stripe_signature_header {
        let signature = stripe_signature_header.unwrap().to_str().ok().unwrap();
        let body = std::str::from_utf8(&body).unwrap();

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
            Ok(webhook_event) => {
                println!("Successfully validated the webhook!!!");
                println!("Web hook type was: {:?}", webhook_event.event_type);
                println!("Web hook object was: {:?}", webhook_event.data.object);
            }
            Err(webhook_error) => {
                println!("Err: {:?}", webhook_error);
            }
        }
    }

    HttpResponse::Ok().json(json!({}))
}
