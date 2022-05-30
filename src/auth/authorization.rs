use crate::auth::token::Claims;
use crate::domain::user_models::UserGroup;

pub fn is_authorized_admin_only(user_id: String, token: Claims) -> bool {
    if (user_id == token.user_id) && token.group == UserGroup::ADMIN.as_str() {
        return true;
    }
    return false;
}

#[cfg(test)]
mod tests {
    use crate::auth::authorization::is_authorized_admin_only;
    use crate::auth::token::Claims;
    use uuid::Uuid;

    #[test]
    fn is_authorized_admin_only_passes() {
        let user_id = Uuid::new_v4().to_string();

        let claims = Claims {
            user_id: user_id.clone(),
            group: "ADMIN".to_string(),
            iss: "".to_string(),
            aud: "".to_string(),
            sub: "".to_string(),
            exp: 0,
            iat: 0,
        };

        assert_eq!(true, is_authorized_admin_only(user_id, claims));
    }

    #[test]
    fn is_authorized_admin_only_does_not_pass() {
        let user_id = Uuid::new_v4().to_string();

        assert_eq!(
            false,
            is_authorized_admin_only(
                user_id.clone(),
                Claims {
                    user_id: user_id.clone(),
                    group: "USER".to_string(),
                    iss: "".to_string(),
                    aud: "".to_string(),
                    sub: "".to_string(),
                    exp: 0,
                    iat: 0,
                }
            )
        );

        assert_eq!(
            false,
            is_authorized_admin_only(
                user_id.clone(),
                Claims {
                    user_id: Uuid::new_v4().to_string(),
                    group: "ADMIN".to_string(),
                    iss: "".to_string(),
                    aud: "".to_string(),
                    sub: "".to_string(),
                    exp: 0,
                    iat: 0,
                }
            )
        );

        assert_eq!(
            false,
            is_authorized_admin_only(
                user_id.clone(),
                Claims {
                    user_id: Uuid::new_v4().to_string(),
                    group: "USER".to_string(),
                    iss: "".to_string(),
                    aud: "".to_string(),
                    sub: "".to_string(),
                    exp: 0,
                    iat: 0,
                }
            )
        );
    }
}
