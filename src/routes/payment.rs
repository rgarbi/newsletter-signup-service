use crate::auth::authorization::is_authorized_user_only;
use actix_web::{web, HttpResponse, Responder};
use serde_json::json;
use sqlx::PgPool;
use tracing::Level;

use crate::auth::token::Claims;
use crate::background::new_subscription_notifier::notify_subscriber;
use crate::background::subscription_history_storer::store_subscription_history_event;
use crate::configuration::get_configuration;
use crate::db::checkout_session_db_broker::{
    insert_checkout_session, retrieve_checkout_session_by_stripe_session_id,
    set_checkout_session_state_to_success_by_stripe_session_id,
};
use crate::db::subscribers_db_broker::{
    retrieve_subscriber_by_id, retrieve_subscriber_by_user_id, set_stripe_customer_id,
};
use crate::db::subscriptions_db_broker::insert_subscription;
use crate::domain::checkout_models::{CreateCheckoutSession, CreateStripeSessionRedirect};
use crate::domain::subscriber_models::OverTheWireSubscriber;
use crate::domain::subscription_history_models::HistoryEventType;
use crate::domain::subscription_models::{
    NewSubscription, OverTheWireCreateSubscription, SubscriptionType,
};
use crate::email_client::EmailClient;
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
    stripe_client: web::Data<StripeClient>,
) -> impl Responder {
    let user_id = user_id.into_inner();
    if !is_authorized_user_only(user_id.clone(), user) {
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
            tracing::event!(Level::ERROR, "Err: {:?}", err);
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

    let lookup_id = get_price_id(new_subscription.subscription_type.clone());

    //Get customer
    let stripe_customer_id = match get_stripe_customer_id(&subscriber, &stripe_client).await {
        Ok(id) => id,
        Err(error_response) => {
            return error_response;
        }
    };

    set_stripe_customer_id(&subscriber.id, stripe_customer_id.as_str(), &pool)
        .await
        .unwrap();

    let price_response = stripe_client.get_stripe_price_by_id(lookup_id).await;
    match price_response {
        Ok(price) => {
            tracing::event!(Level::INFO, "Got prices: {:?}", &price);
            let price_id = price.id.to_string();

            let checkout_session_response = stripe_client
                .create_stripe_checkout_session(
                    price_id,
                    1,
                    stripe_customer_id,
                    success_url,
                    cancel_url,
                    "subscription".to_string(),
                )
                .await;

            match checkout_session_response {
                Ok(checkout_session_created) => {
                    let checkout_session_url = checkout_session_created.url.clone();
                    tracing::event!(
                        Level::INFO,
                        "Checkout session Created!!! URL: {:?}",
                        &checkout_session_created.url
                    );
                    tracing::event!(
                        Level::INFO,
                        "Checkout session Created!!! ID: {:?}",
                        &checkout_session_created.id
                    );

                    let store_checkout_result = insert_checkout_session(
                        user_id.clone(),
                        create_checkout_session.price_lookup_key.clone(),
                        new_subscription,
                        checkout_session_created.id.as_str().to_string(),
                        &pool,
                    )
                    .await;

                    match store_checkout_result {
                        Ok(_) => {
                            tracing::event!(Level::INFO, "REDIRECTING!!!");
                            let redirect_response = CreateStripeSessionRedirect {
                                location: checkout_session_url.as_str().to_string(),
                            };
                            HttpResponse::Ok().json(redirect_response)
                        }
                        Err(err) => {
                            tracing::event!(Level::ERROR, "Err: {:?}", err);
                            HttpResponse::InternalServerError().finish()
                        }
                    }
                }
                Err(stripe_error) => {
                    tracing::event!(Level::ERROR, "Err: {:?}", stripe_error);
                    HttpResponse::InternalServerError().finish()
                }
            }
        }
        Err(err) => {
            tracing::event!(Level::ERROR, "Err: {:?}", err);
            HttpResponse::InternalServerError().finish()
        }
    }
}

#[tracing::instrument(
    name = "Complete Session",
    skip(params, pool, user, stripe_client, email_client),
    fields(
        user_id = %params.0,
        session_id = %params.1,
    )
)]
pub async fn complete_session(
    params: web::Path<(String, String)>,
    pool: web::Data<PgPool>,
    user: Claims,
    stripe_client: web::Data<StripeClient>,
    email_client: web::Data<EmailClient>,
) -> impl Responder {
    let param_tuple: (String, String) = params.into_inner();
    let user_id = param_tuple.clone().0;
    let session_id = param_tuple.clone().1;
    if !is_authorized_user_only(user_id.clone(), user) {
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
            let stripe_session = match stripe_client
                .get_stripe_session(checkout.stripe_session_id.clone())
                .await
            {
                Ok(session) => session,
                Err(err) => {
                    tracing::event!(
                        Level::ERROR,
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
                Ok(subscription) => {
                    if transaction.commit().await.is_err() {
                        HttpResponse::InternalServerError().finish()
                    } else {
                        store_subscription_history_event(
                            subscription.id,
                            HistoryEventType::Created,
                            &pool,
                        );

                        notify_subscriber(subscription.id, &email_client, &pool).await;

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
            tracing::event!(Level::ERROR, "Err: {:?}", err);
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
    let user_id = user_id.into_inner();
    if !is_authorized_user_only(user_id.clone(), user) {
        return HttpResponse::Unauthorized().finish();
    }

    let subscriber = match retrieve_subscriber_by_user_id(user_id.as_str(), &pool).await {
        Ok(subscriber) => subscriber,
        Err(err) => {
            tracing::event!(Level::ERROR, "Err: {:?}", err);
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
            tracing::event!(
                Level::INFO,
                "Got the following back {:?}",
                &stripe_billing_portal_session.url
            );
            let redirect_response = CreateStripeSessionRedirect {
                location: stripe_billing_portal_session.url,
            };
            HttpResponse::Ok().json(redirect_response)
        }
        Err(err) => {
            tracing::event!(
                Level::ERROR,
                "Something blew up when creating the portal session! {:?}",
                err
            );
            HttpResponse::InternalServerError().finish()
        }
    };
}

async fn get_stripe_customer_id(
    subscriber: &OverTheWireSubscriber,
    client: &StripeClient,
) -> Result<String, HttpResponse> {
    if subscriber.stripe_customer_id.is_none() {
        return match client
            .create_stripe_customer(subscriber.email_address.clone())
            .await
        {
            Ok(customer) => Ok(customer.id),
            Err(err) => {
                tracing::event!(Level::ERROR, "Err: {:?}", err);
                Err(HttpResponse::InternalServerError().finish())
            }
        };
    } else {
        Ok(subscriber.stripe_customer_id.clone().unwrap())
    }
}

fn get_price_id(subscription_type: SubscriptionType) -> String {
    let config = get_configuration().unwrap().stripe_client;
    match subscription_type {
        SubscriptionType::Digital => config.digital_price_id,
        SubscriptionType::Paper => config.paper_price_id,
    }
}

#[cfg(test)]
mod tests {
    use crate::configuration::get_configuration;
    use crate::domain::subscription_models::SubscriptionType;
    use crate::routes::payment::get_price_id;

    #[test]
    fn get_price_id_works() {
        let config = get_configuration().unwrap().stripe_client;

        assert_eq!(config.paper_price_id, get_price_id(SubscriptionType::Paper));
        assert_eq!(
            config.digital_price_id,
            get_price_id(SubscriptionType::Digital)
        );
    }
}
