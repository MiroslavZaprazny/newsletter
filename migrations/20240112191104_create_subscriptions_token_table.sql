CREATE TABLE subscription_tokens(
    subscription_token TEXT NOT NULL,
    subscriber_id uuid NOT NULL,
    CONSTRAINT fk_subscriber_id
      FOREIGN KEY(subscriber_id) 
	  REFERENCES subscriptions(id),
    PRIMARY KEY (subscription_token)
);
