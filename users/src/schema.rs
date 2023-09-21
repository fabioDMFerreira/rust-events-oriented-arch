use diesel::table;

table! {
  users (id) {
      id -> Uuid,
      name -> Text,
      password -> Text,
  }
}
