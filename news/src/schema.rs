use diesel::table;

table! {
  feeds (id) {
      id -> Uuid,
      author -> Text,
      title -> Text,
      url -> Text,
  }
}

table! {
  news (id) {
      id -> Uuid,
      author -> Text,
      url -> Text,
      title -> Text,
      publish_date -> Nullable<Date>,
      feed_id -> Uuid,
  }
}

table! {
  subscriptions (feed_id, user_id) {
      feed_id -> Uuid,
      user_id -> Uuid,
  }
}
