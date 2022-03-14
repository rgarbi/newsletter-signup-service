use std::collections::HashSet;
use std::future::Future;
use std::pin::Pin;

use actix_web::http::{StatusCode, Uri};
use actix_web::{Error, FromRequest, HttpResponse, ResponseError};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use actix_web_httpauth::headers::www_authenticate::bearer::Bearer;
use cached::proc_macro::once;
use derive_more::Display;
use jsonwebtoken::jwk::{AlgorithmParameters, JwkSet};
use jsonwebtoken::{decode, decode_header, Algorithm, DecodingKey, Validation};
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::configuration::{get_configuration, Auth0Config};

#[derive(Debug, Display)]
enum ClientError {
    #[display(fmt = "authentication")]
    Authentication(actix_web_httpauth::extractors::AuthenticationError<Bearer>),
    #[display(fmt = "decode")]
    Decode(jsonwebtoken::errors::Error),
    #[display(fmt = "not_found")]
    NotFound(String),
    #[display(fmt = "unsupported_algorithm")]
    UnsupportedAlgortithm(AlgorithmParameters),
}

#[derive(Serialize)]
pub struct ErrorMessage {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_description: Option<String>,
    pub message: String,
}

impl ResponseError for ClientError {
    fn status_code(&self) -> StatusCode {
        StatusCode::UNAUTHORIZED
    }

    fn error_response(&self) -> HttpResponse {
        match self {
            Self::Authentication(_) => HttpResponse::Unauthorized().json(ErrorMessage {
                error: None,
                error_description: None,
                message: "Requires authentication".to_string(),
            }),
            Self::Decode(_) => HttpResponse::Unauthorized().json(ErrorMessage {
                error: Some("invalid_token".to_string()),
                error_description: Some(
                    "Authorization header value must follow this format: Bearer access-token"
                        .to_string(),
                ),
                message: "Bad credentials".to_string(),
            }),
            Self::NotFound(msg) => HttpResponse::Unauthorized().json(ErrorMessage {
                error: Some("invalid_token".to_string()),
                error_description: Some(msg.to_string()),
                message: "Bad credentials".to_string(),
            }),
            Self::UnsupportedAlgortithm(alg) => HttpResponse::Unauthorized().json(ErrorMessage {
                error: Some("invalid_token".to_string()),
                error_description: Some(format!(
                    "Unsupported encryption algortithm expected RSA got {:?}",
                    alg
                )),
                message: "Bad credentials".to_string(),
            }),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct User {
    #[serde(alias = "sub")]
    pub user_id: String,
    pub permissions: Option<HashSet<String>>,
}

impl User {
    pub fn validate_permissions(&self, required_permissions: &HashSet<String>) -> bool {
        self.permissions.as_ref().map_or(false, |permissions| {
            permissions.is_superset(required_permissions)
        })
    }
}

impl FromRequest for User {
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self, Self::Error>>>>;

    fn from_request(
        req: &actix_web::HttpRequest,
        _payload: &mut actix_web::dev::Payload,
    ) -> Self::Future {
        let config = get_configuration().unwrap().auth0config;
        let extractor = BearerAuth::extract(req);
        Box::pin(async move {
            let credentials = extractor.await.map_err(ClientError::Authentication)?;
            let token = credentials.token();
            println!("Raw Cred: {:?}", credentials);
            println!("Raw Token: {}", token);
            let header = decode_header(token).map_err(ClientError::Decode)?;
            let kid = header.kid.ok_or_else(|| {
                ClientError::NotFound("kid not found in token header".to_string())
            })?;

            let jwks: JwkSet = get_jwks(config.clone()).await;
            let jwk = jwks
                .find(&kid)
                .ok_or_else(|| ClientError::NotFound("No JWK found for kid".to_string()))?;
            match jwk.clone().algorithm {
                AlgorithmParameters::RSA(ref rsa) => {
                    let mut validation = Validation::new(Algorithm::RS256);
                    validation.set_audience(&[config.audience]);
                    validation.set_issuer(&[Uri::builder()
                        .scheme("https")
                        .authority(config.domain)
                        .path_and_query("/")
                        .build()
                        .unwrap()]);
                    let key = DecodingKey::from_rsa_components(&rsa.n, &rsa.e)
                        .map_err(ClientError::Decode)?;
                    let token =
                        decode::<User>(token, &key, &validation).map_err(ClientError::Decode)?;
                    println!("Token: {:?}", token);
                    Ok(token.claims)
                }
                algorithm => Err(ClientError::UnsupportedAlgortithm(algorithm).into()),
            }
        })
    }
}

#[once(time = 1000)]
pub async fn get_jwks(config: Auth0Config) -> JwkSet {
    let domain = config.domain.as_str();
    println!("Started jwks call");
    let response = Client::new()
        .get(
            Uri::builder()
                .scheme("https")
                .authority(domain)
                .path_and_query("/.well-known/jwks.json")
                .build()
                .unwrap()
                .to_string(),
        )
        .send()
        .await
        .expect("Failed to get the signing public keys");
    println!("Done with jwks call - THIS IS GONNA BE SUPER SLOW LET'S CACHE THE KEYS");
    let response_body = response.text().await.unwrap();
    serde_json::from_str(response_body.as_str()).unwrap()
}
