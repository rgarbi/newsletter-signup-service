use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize, Serialize, Debug)]
pub struct User {
    pub user_id: Uuid,
    pub email_address: String,
    pub password: String,
    pub user_group: UserGroup,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct SignUp {
    pub email_address: String,
    pub password: String,
    pub name: String,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct LogIn {
    pub email_address: String,
    pub password: String,
}

#[derive(Deserialize, Serialize)]
pub struct ResetPassword {
    pub email_address: String,
    pub old_password: String,
    pub new_password: String,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct ResetPasswordFromForgotPassword {
    pub user_id: String,
    pub new_password: String,
}

#[derive(Deserialize, Serialize)]
pub struct ForgotPassword {
    pub email_address: String,
}

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, Eq)]
pub enum UserGroup {
    USER,
    ADMIN,
}

impl UserGroup {
    pub fn as_str(&self) -> &'static str {
        match self {
            UserGroup::USER => "USER",
            UserGroup::ADMIN => "ADMIN",
        }
    }
}

impl SignUp {
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).expect("Was not able to serialize.")
    }
}

impl ResetPassword {
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).expect("Was not able to serialize.")
    }
}

impl LogIn {
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).expect("Was not able to serialize.")
    }
}

impl ForgotPassword {
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).expect("Was not able to serialize.")
    }
}

impl ResetPasswordFromForgotPassword {
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).expect("Was not able to serialize.")
    }
}

pub fn from_str_to_user_group(val: String) -> UserGroup {
    if val.eq("USER") {
        return UserGroup::USER;
    }

    if val.eq("ADMIN") {
        return UserGroup::ADMIN;
    }

    tracing::error!("Could not map string: {} to the enum UserGroup", val);
    UserGroup::USER
}

#[cfg(test)]
mod tests {
    use crate::domain::user_models::{from_str_to_user_group, UserGroup};

    #[test]
    fn user_group_as_str() {
        let user = UserGroup::USER.as_str();
        let admin = UserGroup::ADMIN.as_str();

        assert_eq!("USER", user);
        assert_eq!("ADMIN", admin);
    }

    #[test]
    fn from_str_to_user_group_test() {
        assert_eq!(
            UserGroup::USER,
            from_str_to_user_group(String::from(UserGroup::USER.as_str()))
        );
        assert_eq!(
            UserGroup::ADMIN,
            from_str_to_user_group(String::from(UserGroup::ADMIN.as_str()))
        );
        assert_eq!(
            UserGroup::USER,
            from_str_to_user_group(String::from("not a user group"))
        );
    }
}
