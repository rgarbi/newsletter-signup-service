use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};

pub fn get_argon() -> Argon2<'static> {
    Argon2::default()
}

pub async fn hash_password(password: String) -> String {
    tokio::task::spawn_blocking(move || {
        let mut salt_bytes = [0u8; 16];
        getrandom::fill(&mut salt_bytes).unwrap();
        let salt = SaltString::encode_b64(&salt_bytes).unwrap();

        let argon2 = get_argon();
        let hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .unwrap()
            .to_string();
        hash
    })
    .await
    .unwrap()
}

pub async fn validate_password(password: String, hashed_password: String) -> bool {
    tokio::task::spawn_blocking(move || {
        let parsed_hash = PasswordHash::new(&hashed_password).unwrap();
        get_argon()
            .verify_password(password.as_bytes(), &parsed_hash)
            .is_ok()
    })
    .await
    .unwrap()
}

#[cfg(test)]
mod tests {
    use std::time::Instant;

    use rand::distr::Alphanumeric;
    use rand::{rng, RngExt};
    use uuid::Uuid;

    use crate::auth::password_hashing::{hash_password, validate_password};

    #[tokio::test]
    async fn given_a_random_password_i_can_hash_it_and_compare_it() {
        let password: String = rng()
            .sample_iter(Alphanumeric)
            .take(100)
            .map(char::from)
            .collect();
        let hashed_password = hash_password(password.clone()).await;

        println!("Hashed: {}", hashed_password);

        assert_eq!(true, validate_password(password, hashed_password).await)
    }

    #[tokio::test]
    async fn given_a_random_password_i_can_hash_it_many_times() {
        for _ in 1..20 {
            let start = Instant::now();
            let password = Uuid::new_v4().to_string();
            hash_password(password.clone()).await;
            let elapsed = start.elapsed();
            println!("Done: {}", elapsed.as_millis())
        }
    }
}
