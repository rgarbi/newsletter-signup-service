#[cfg(test)]
pub mod test {
    use super::super::rocket;
    use rocket::http::Status;
    use rocket::local::blocking::Client;

    #[test]
    fn get_subscriber_by_id_test() {
        let client = Client::tracked(rocket()).expect("valid rocket instance");
        let response = client.get("/subscribers/1234").dispatch();
        assert_eq!(response.status(), Status::Ok);
    }

    #[test]
    fn get_subscription_by_id_test() {
        let client = Client::tracked(rocket()).expect("valid rocket instance");
        let response = client.get("/subscriptions/1234").dispatch();
        assert_eq!(response.status(), Status::Ok);
    }
}
