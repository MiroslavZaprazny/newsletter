use crate::domain::{Email, SubscriberName};

pub struct Subscriber {
    pub name: SubscriberName,
    pub email: Email,
}
