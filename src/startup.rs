use std::net::TcpListener;

use actix_cors::Cors;
use actix_web::dev::Server;
use actix_web::http::header;
use actix_web::middleware::DefaultHeaders;
use actix_web::web::Data;
use actix_web::{web, App, HttpServer};
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use tracing_actix_web::TracingLogger;

use crate::configuration::{DatabaseSettings, Settings};
use crate::email_client::EmailClient;
use crate::routes;

pub struct Application {
    port: u16,
    server: Server,
}

impl Application {
    // We have converted the `build` function into a constructor for
    // `Application`.
    pub async fn build(configuration: Settings) -> Result<Self, std::io::Error> {
        let connection_pool = get_connection_pool(&configuration.database);

        let timeout = configuration.email_client.timeout();
        let sender_email = configuration
            .email_client
            .sender()
            .expect("Invalid sender email address.");
        let email_client = EmailClient::new(
            configuration.email_client.base_url,
            sender_email,
            configuration.email_client.api_key,
            timeout,
        );

        let address = format!(
            "{}:{}",
            configuration.application.host, configuration.application.port
        );
        let listener = TcpListener::bind(&address)?;
        let port = listener.local_addr().unwrap().port();

        let server = run(listener, connection_pool, email_client)?;
        Ok(Self { port, server })
    }
    pub fn port(&self) -> u16 {
        self.port
    }

    pub async fn run_until_stopped(self) -> Result<(), std::io::Error> {
        self.server.await
    }
}

pub fn get_connection_pool(configuration: &DatabaseSettings) -> PgPool {
    PgPoolOptions::new()
        .connect_timeout(std::time::Duration::from_secs(2))
        .connect_lazy_with(configuration.with_db())
}

pub fn security_headers() -> DefaultHeaders {
    DefaultHeaders::new()
        .add((header::X_XSS_PROTECTION, "0"))
        .add((
            header::STRICT_TRANSPORT_SECURITY,
            "max-age=31536000; includeSubDomains",
        ))
        .add((header::X_FRAME_OPTIONS, "deny"))
        .add((header::X_CONTENT_TYPE_OPTIONS, "nosniff"))
        .add((
            header::CONTENT_SECURITY_POLICY,
            "default-src 'self'; frame-ancestors 'none';",
        ))
        .add((
            header::CACHE_CONTROL,
            "no-cache, no-store, max-age=0, must-revalidate",
        ))
        .add((header::PRAGMA, "no-cache"))
        .add((header::EXPIRES, "0"))
        .add((header::ACCESS_CONTROL_ALLOW_ORIGIN, "*"))
        .add((header::ACCESS_CONTROL_ALLOW_HEADERS, "*"))
        .add((
            header::ACCESS_CONTROL_ALLOW_METHODS,
            "OPTIONS,GET,PUT,POST,DELETE,HEAD",
        ))
}

pub fn define_cors() -> Cors {
    Cors::default()
        .allow_any_origin()
        .allowed_methods(vec!["GET", "POST", "PUT", "DELETE"])
        .allowed_headers(vec![header::AUTHORIZATION, header::ACCEPT])
        .allowed_header(header::CONTENT_TYPE)
        .max_age(3600)
}

pub fn run(
    listener: TcpListener,
    connection: PgPool,
    email_client: EmailClient,
) -> Result<Server, std::io::Error> {
    let connection = web::Data::new(connection);
    let email_client = Data::new(email_client);
    let server = HttpServer::new(move || {
        App::new()
            .wrap(define_cors())
            .wrap(security_headers())
            .wrap(TracingLogger::default())
            .route("/sign_up", web::post().to(routes::sign_up))
            .route(
                "/check_token/{user_id}",
                web::post().to(routes::check_token),
            )
            .route("/login", web::post().to(routes::login))
            .route("/forgot_password", web::post().to(routes::forgot_password))
            .route(
                "/forgot_password/otp/{otp}",
                web::get().to(routes::forgot_password_login),
            )
            .route(
                "/forgot_password/reset_password",
                web::post().to(routes::reset_password_from_forgot_password),
            )
            .route("/reset_password", web::post().to(routes::reset_password))
            .route("/health_check", web::get().to(routes::health_check))
            .route(
                "/subscriptions/{id}",
                web::get().to(routes::get_subscription_by_id),
            )
            .route(
                "/subscriptions/{id}",
                web::delete().to(routes::cancel_subscription_by_id),
            )
            .route("/subscribers", web::post().to(routes::post_subscriber))
            .route(
                "/subscribers",
                web::get().to(routes::get_subscriber_by_query),
            )
            .route(
                "/subscribers/{id}",
                web::get().to(routes::get_subscriber_by_id),
            )
            .route(
                "/subscribers/{id}/subscriptions",
                web::get().to(routes::get_subscriptions_by_subscriber_id),
            )
            .route(
                "/checkout/{user_id}",
                web::post().to(routes::create_checkout_session),
            )
            .route(
                "/checkout/{user_id}/session/{session_id}",
                web::post().to(routes::complete_session),
            )
            .route(
                "/checkout/{user_id}/manage",
                web::post().to(routes::create_stripe_portal_session),
            )
            .route("/webhook", web::post().to(routes::handle_webhook))
            .app_data(connection.clone())
            .app_data(email_client.clone())
    })
    .listen(listener)?
    .run();

    Ok(server)
}
