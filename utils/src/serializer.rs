use chrono::NaiveDate;

pub fn time_to_json(t: NaiveDate) -> String {
    t.to_string()
}

pub mod serde_naive_date {
    use super::*;
    use serde::{de::Error, Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S: Serializer>(
        time: &Option<NaiveDate>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        match time {
            Some(time) => time_to_json(*time).serialize(serializer),
            None => "".serialize(serializer),
        }
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<Option<NaiveDate>, D::Error> {
        let time: &str = Deserialize::deserialize(deserializer)?;
        if !time.is_empty() {
            Ok(Some(
                NaiveDate::parse_from_str(time, "%Y-%m-%d").map_err(D::Error::custom)?,
            ))
        } else {
            Ok(None)
        }
    }
}
