use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use rand_core::OsRng;

pub fn hash_password(password: String) -> String {
    let salt = SaltString::generate(&mut OsRng);

    let argon2 = Argon2::default();
    let hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .unwrap()
        .to_string();
    hash
}

pub fn validate_password(password: String, hashed_password: String) -> bool {
    let parsed_hash = PasswordHash::new(&hashed_password).unwrap();
    Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok()
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use crate::auth::password_hashing::{hash_password, validate_password};

    #[test]
    fn given_a_random_password_i_can_hash_it_and_compare_it() {
        let password = Uuid::new_v4().to_string();
        let hashed_password = hash_password(password.clone());

        println!("Hashed: {}", hashed_password);

        assert_eq!(true, validate_password(password, hashed_password))
    }
}
