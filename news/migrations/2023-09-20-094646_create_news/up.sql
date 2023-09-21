CREATE TABLE news (
    id UUID PRIMARY KEY,
    author VARCHAR(100),
    url VARCHAR(255) NOT NULL,
    title VARCHAR(255) NOT NULL,
    publish_date DATE,
    feed_id UUID NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (feed_id) REFERENCES feeds(id),
    CONSTRAINT uc_news UNIQUE (feed_id, title)
);
