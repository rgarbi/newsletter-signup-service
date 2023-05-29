use std::str::FromStr;

use actix_web::{web, HttpResponse};
use chrono::Datelike;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use uuid::Uuid;

pub fn from_path_to_uuid(id: &web::Path<String>) -> Result<Uuid, HttpResponse> {
    match Uuid::from_str(id.as_str()) {
        Ok(uuid) => Ok(uuid),
        Err(_) => {
            tracing::error!("Got a malformed UUID");
            Err(HttpResponse::BadRequest().finish())
        }
    }
}

pub fn from_string_to_uuid(id: &str) -> Result<Uuid, HttpResponse> {
    match Uuid::from_str(id) {
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

trait NaiveDateExt {
    fn days_in_month(&self) -> i32;
    fn days_in_year(&self) -> i32;
    fn is_leap_year(&self) -> bool;
}

impl NaiveDateExt for chrono::NaiveDate {
    fn days_in_month(&self) -> i32 {
        let month = self.month();
        match month {
            1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
            4 | 6 | 9 | 11 => 30,
            2 => if self.is_leap_year() { 29 } else { 28 },
            _ => panic!("Invalid month: {}", month),
        }
    }

    fn days_in_year(&self) -> i32 {
        if self.is_leap_year() { 366 } else { 365 }
    }

    fn is_leap_year(&self) -> bool {
        let year = self.year();
        return year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
    }
}

#[cfg(test)]
mod tests {
    use actix_web::web::Path;
    use chrono::NaiveDate;
    use uuid::Uuid;

    use crate::util::{from_path_to_uuid, from_string_to_uuid, generate_random_token, NaiveDateExt};

    #[test]
    fn native_date_ext_days_in_month_test() {
        assert_eq!(NaiveDate::parse_from_str("2004-01-01", "%Y-%m-%d").unwrap().is_leap_year(), true);
        assert_eq!(NaiveDate::parse_from_str("2004-01-01", "%Y-%m-%d").unwrap().days_in_year(), 365);
        assert_eq!(NaiveDate::parse_from_str("2005-01-01", "%Y-%m-%d").unwrap().days_in_year(), 364);
        assert_eq!(NaiveDate::parse_from_str("2004-01-01", "%Y-%m-%d").unwrap().days_in_month(), 31);
        assert_eq!(NaiveDate::parse_from_str("2004-02-01", "%Y-%m-%d").unwrap().days_in_month(), 29);
        assert_eq!(NaiveDate::parse_from_str("2005-02-01", "%Y-%m-%d").unwrap().days_in_month(), 28);
        assert_eq!(NaiveDate::parse_from_str("2004-03-01", "%Y-%m-%d").unwrap().days_in_month(), 31);
        assert_eq!(NaiveDate::parse_from_str("2004-04-01", "%Y-%m-%d").unwrap().days_in_month(), 30);
        assert_eq!(NaiveDate::parse_from_str("2004-05-01", "%Y-%m-%d").unwrap().days_in_month(), 31);
        assert_eq!(NaiveDate::parse_from_str("2004-06-01", "%Y-%m-%d").unwrap().days_in_month(), 30);
        assert_eq!(NaiveDate::parse_from_str("2004-07-01", "%Y-%m-%d").unwrap().days_in_month(), 31);
        assert_eq!(NaiveDate::parse_from_str("2004-08-01", "%Y-%m-%d").unwrap().days_in_month(), 31);
        assert_eq!(NaiveDate::parse_from_str("2004-09-01", "%Y-%m-%d").unwrap().days_in_month(), 30);
        assert_eq!(NaiveDate::parse_from_str("2004-10-01", "%Y-%m-%d").unwrap().days_in_month(), 31);
        assert_eq!(NaiveDate::parse_from_str("2004-11-01", "%Y-%m-%d").unwrap().days_in_month(), 30);
        assert_eq!(NaiveDate::parse_from_str("2004-12-01", "%Y-%m-%d").unwrap().days_in_month(), 31);
    }

    #[test]
    fn a_uuid_is_valid() {
        let uuid = Uuid::new_v4();

        assert_eq!(
            uuid,
            from_path_to_uuid(&Path::try_from(uuid.to_string()).unwrap()).unwrap()
        );

        assert_eq!(uuid, from_string_to_uuid(&uuid.to_string()).unwrap());
    }

    #[test]
    fn generate_random_token_test() {
        let value = generate_random_token();
        assert!(value.len() > 0);
    }

    #[quickcheck_macros::quickcheck]
    fn anything_not_a_uuid_is_invalid(invalid_uuid: String) -> bool {
        from_path_to_uuid(&Path::try_from(invalid_uuid).unwrap()).is_err()
    }

    #[quickcheck_macros::quickcheck]
    fn anything_not_a_uuid_is_invalid_from_string(invalid_uuid: String) -> bool {
        from_string_to_uuid(&Path::try_from(invalid_uuid).unwrap()).is_err()
    }
}
