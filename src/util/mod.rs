use std::str::FromStr;

use actix_web::{web, HttpResponse};
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use uuid::Uuid;

pub fn from_string_to_uuid(id: &web::Path<String>) -> Result<Uuid, HttpResponse> {
    match Uuid::from_str(id.as_str().clone()) {
        Ok(uuid) => Ok(uuid),
        Err(_) => {
            tracing::error!("Got a malformed UUID");
            Err(HttpResponse::BadRequest().finish())
        }
    }
}

pub fn standardize_email(email: &str) -> String {
    email.to_string().to_lowercase()
}

pub fn generate_random_token() -> String {
    let mut rng = thread_rng();
    std::iter::repeat_with(|| rng.sample(Alphanumeric))
        .map(char::from)
        .take(50)
        .collect()
}

#[cfg(test)]
mod tests {
    use actix_web::web::Path;
    use uuid::Uuid;

    use crate::util::from_string_to_uuid;

    #[test]
    fn a_uuid_is_valid() {
        let uuid = Uuid::new_v4();

        assert_eq!(
            uuid,
            from_string_to_uuid(&Path::try_from(uuid.to_string()).unwrap()).unwrap()
        );
    }

    #[quickcheck_macros::quickcheck]
    fn anything_not_a_uuid_is_invalid(invalid_uuid: String) -> bool {
        from_string_to_uuid(&Path::try_from(invalid_uuid).unwrap()).is_err()
    }
}
