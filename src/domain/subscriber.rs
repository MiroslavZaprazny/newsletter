use crate::domain::{Email, SubscriberName};

#[derive(Debug)]
pub struct Subscriber {
    pub name: SubscriberName,
    pub email: Email,
}
