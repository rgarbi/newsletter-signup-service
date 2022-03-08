use std::str::FromStr;

use actix_web::{web, HttpResponse};
use uuid::Uuid;

pub fn from_string_to_uuid(id: web::Path<String>) -> Result<Uuid, HttpResponse> {
    match Uuid::from_str(id.into_inner().as_str()) {
        Ok(uuid) => Ok(uuid),
        Err(_) => {
            tracing::error!("Got a malformed UUID");
            Err(HttpResponse::BadRequest().finish())
        }
    }
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
            from_string_to_uuid(Path::try_from(uuid.to_string()).unwrap()).unwrap()
        );
    }

    #[quickcheck_macros::quickcheck]
    fn anything_not_a_uuid_is_invalid(invalid_uuid: String) -> bool {
        from_string_to_uuid(Path::try_from(invalid_uuid).unwrap()).is_err()
    }
}