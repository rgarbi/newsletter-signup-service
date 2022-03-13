use actix_web::http::header::{HeaderMap, HeaderValue};
use actix_web::http::{header, StatusCode};
use actix_web::{HttpRequest, HttpResponse};
use anyhow::Context;
use secrecy::Secret;

pub struct Credentials {
    username: String,
    password: Secret<String>,
}

fn basic_authentication(headers: &HeaderMap) -> Result<Credentials, anyhow::Error> {
    let header_value = headers
        .get("Authorization")
        .context("The 'Authorization' header was missing")?
        .to_str()
        .context("The 'Authorization' header was not a valid UTF8 string.")?;
    let base64encoded_segment = header_value
        .strip_prefix("Basic ")
        .context("The authorization scheme was not 'Basic'.")?;
    let decoded_bytes = base64::decode_config(base64encoded_segment, base64::STANDARD)
        .context("Failed to base64-decode 'Basic' credentials.")?;
    let decoded_credentials = String::from_utf8(decoded_bytes)
        .context("The decoded credential string is not valid UTF8.")?;
    // Split into two segments, using ':' as delimitator
    let mut credentials = decoded_credentials.splitn(2, ':');
    let username = credentials
        .next()
        .ok_or_else(|| anyhow::anyhow!("A username must be provided in 'Basic' auth."))?
        .to_string();
    let password = credentials
        .next()
        .ok_or_else(|| anyhow::anyhow!("A password must be provided in 'Basic' auth."))?
        .to_string();
    Ok(Credentials {
        username,
        password: Secret::new(password),
    })
}

pub fn auth(request: HttpRequest) -> Result<Credentials, HttpResponse> {
    match basic_authentication(request.headers()) {
        Ok(credentials) => Ok(credentials),
        Err(_) => {
            let mut response = HttpResponse::new(StatusCode::UNAUTHORIZED);
            let header_value = HeaderValue::from_str(r#"Basic realm="subscribe""#).unwrap();
            response
                .headers_mut()
                // actix_web::http::header provides a collection of constants
                // for the names of several well-known/standard HTTP headers
                .insert(header::WWW_AUTHENTICATE, header_value);
            Err(response)
        }
    }
}
