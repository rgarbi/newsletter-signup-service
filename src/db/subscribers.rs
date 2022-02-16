use crate::models::subscriber::Subscriber;

pub async fn find_subscriber_by_id(id: i64) -> Option<Subscriber> {
    Option::from(Subscriber::new_subscriber(
        Option::from(id),
        String::from("first"),
        String::from("last"),
        String::from("first.last@somemail.com"),
    ))
}

pub async fn store_subscriber(subscriber: Subscriber) -> Subscriber {
    subscriber
}
