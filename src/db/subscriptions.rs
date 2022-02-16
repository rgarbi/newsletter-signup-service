use crate::models::subscription::Subscription;

pub async fn find_subscriptions_by_id(_id: i64) -> Option<Subscription> {
    Option::from(Subscription::new_subscription())
}
