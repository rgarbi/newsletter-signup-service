use argon2::password_hash::SaltString;
use argon2::{Algorithm, Argon2, Params, PasswordHash, PasswordHasher, PasswordVerifier, Version};
use rand_core::OsRng;

pub fn get_argon() -> Argon2<'static> {
    Argon2::new(
        Algorithm::Argon2id,
        Version::V0x13,
        Params::new(15000, 2, 1, Option::from(64)).unwrap(),
    )
}

pub fn hash_password(password: String) -> String {
    let salt = SaltString::generate(&mut OsRng);

    let argon2 = get_argon();
    let hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .unwrap()
        .to_string();
    hash
}

pub fn validate_password(password: String, hashed_password: String) -> bool {
    let parsed_hash = PasswordHash::new(&hashed_password).unwrap();
    get_argon()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok()
}

#[cfg(test)]
mod tests {
    use rand::distributions::Alphanumeric;
    use rand::{thread_rng, Rng};
    use std::time::Instant;
    use uuid::Uuid;

    use crate::auth::password_hashing::{hash_password, validate_password};

    #[test]
    fn given_a_random_password_i_can_hash_it_and_compare_it() {
        let password: String = thread_rng()
            .sample_iter(&Alphanumeric)
            .take(100)
            .map(char::from)
            .collect();
        let hashed_password = hash_password(password.clone());

        println!("Hashed: {}", hashed_password);

        assert_eq!(true, validate_password(password, hashed_password))
    }

    #[test]
    fn given_a_random_password_i_can_hash_it_many_times() {
        for _ in 1..20 {
            let start = Instant::now();
            let password = Uuid::new_v4().to_string();
            hash_password(password.clone());
            let elapsed = start.elapsed();
            println!("Done: {}", elapsed.as_millis())
        }
    }
}
