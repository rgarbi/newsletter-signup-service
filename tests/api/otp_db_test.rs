use chrono::{Duration, Utc};
use claims::{assert_err, assert_ok};
use uuid::Uuid;

use newsletter_signup_service::db::otp_db_broker::{
    get_otp_by_otp, insert_otp, set_to_used_by_otp,
};
use newsletter_signup_service::domain::otp_models::OneTimePasscode;

use crate::helper::spawn_app;

#[tokio::test]
async fn insert_otp_works() {
    let app = spawn_app().await;

    let otp = OneTimePasscode {
        id: Uuid::new_v4(),
        user_id: Uuid::new_v4().to_string(),
        one_time_passcode: Uuid::new_v4().to_string(),
        issued_on: Utc::now(),
        expires_on: Utc::now() + Duration::seconds(100),
        used: false,
    };

    let result = insert_otp(otp, &app.db_pool).await;
    assert_ok!(result);
}

#[tokio::test]
async fn insert_duplicate_otp_blows_up() {
    let app = spawn_app().await;

    let otp = OneTimePasscode {
        id: Uuid::new_v4(),
        user_id: Uuid::new_v4().to_string(),
        one_time_passcode: Uuid::new_v4().to_string(),
        issued_on: Utc::now(),
        expires_on: Utc::now() + Duration::seconds(100),
        used: false,
    };

    let _discard = insert_otp(otp.clone(), &app.db_pool).await;
    let result = insert_otp(otp.clone(), &app.db_pool).await;
    assert_err!(result);
}

#[tokio::test]
async fn get_otp_by_otp_works() {
    let app = spawn_app().await;

    let otp = OneTimePasscode {
        id: Uuid::new_v4(),
        user_id: Uuid::new_v4().to_string(),
        one_time_passcode: Uuid::new_v4().to_string(),
        issued_on: Utc::now(),
        expires_on: Utc::now() + Duration::seconds(100),
        used: false,
    };

    let result = insert_otp(otp.clone(), &app.db_pool).await;
    assert_ok!(result);

    let saved = get_otp_by_otp(otp.one_time_passcode.as_str(), &app.db_pool).await;
    assert_ok!(&saved);
    assert_eq!(otp.one_time_passcode, saved.unwrap().one_time_passcode);
}

#[tokio::test]
async fn get_otp_by_otp_blows_up_when_not_found() {
    let app = spawn_app().await;

    let saved = get_otp_by_otp(Uuid::new_v4().to_string().as_str(), &app.db_pool).await;
    assert_err!(saved);
}

#[tokio::test]
async fn set_to_used_by_otp_works() {
    let app = spawn_app().await;

    let otp = OneTimePasscode {
        id: Uuid::new_v4(),
        user_id: Uuid::new_v4().to_string(),
        one_time_passcode: Uuid::new_v4().to_string(),
        issued_on: Utc::now(),
        expires_on: Utc::now() + Duration::seconds(100),
        used: false,
    };

    let result = insert_otp(otp.clone(), &app.db_pool).await;
    assert_ok!(result);

    let saved = get_otp_by_otp(otp.clone().one_time_passcode.as_str(), &app.db_pool).await;
    assert_ok!(&saved);
    assert_eq!(
        otp.one_time_passcode.clone(),
        saved.unwrap().one_time_passcode
    );

    let set_to_used_result = set_to_used_by_otp(otp.one_time_passcode.as_str(), &app.db_pool).await;
    assert_ok!(set_to_used_result);

    let used_result = get_otp_by_otp(otp.clone().one_time_passcode.as_str(), &app.db_pool).await;
    assert_ok!(&used_result);

    assert_eq!(true, used_result.unwrap().used);
}
