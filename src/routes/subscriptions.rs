use actix_web::{HttpResponse, Responder};

pub async fn subscription() -> impl Responder {
    HttpResponse::Ok()
}
