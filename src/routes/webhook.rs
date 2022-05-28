use actix_web::{web, HttpRequest, HttpResponse, Responder};
use serde_json::json;
use sqlx::PgPool;
use tracing::Level;

use crate::configuration::get_configuration;

#[tracing::instrument(name = "Handle Webhook", skip(req, body, _pool), fields())]
pub async fn handle_webhook(
    req: HttpRequest,
    body: web::Bytes,
    _pool: web::Data<PgPool>,
) -> impl Responder {
    let _configuration = get_configuration().unwrap();
    let stripe_signature_header = req.headers().get("Stripe-Signature");

    if let Some(..) = stripe_signature_header {
        let _signature = stripe_signature_header.unwrap().to_str().ok().unwrap();
        let _body = std::str::from_utf8(&body).unwrap();

        tracing::event!(Level::INFO, "Got a webhook")
    }

    HttpResponse::Ok().json(json!({}))
}
