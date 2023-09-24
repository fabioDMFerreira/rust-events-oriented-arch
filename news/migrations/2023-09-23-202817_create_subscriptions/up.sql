CREATE TABLE subscriptions (
    feed_id UUID NOT NULL,
    user_id UUID NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (feed_id) REFERENCES feeds (id),
    PRIMARY KEY (feed_id, user_id)
);
