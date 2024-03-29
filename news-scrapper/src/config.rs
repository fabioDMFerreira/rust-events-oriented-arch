#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub logs_path: String,
    pub kafka_url: String,
}

impl Config {
    pub fn init() -> Config {
        let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let logs_path = std::env::var("LOGS_PATH").unwrap_or_else(|_| String::from(""));
        let kafka_url = std::env::var("KAFKA_URL").expect("KAFKA_URL must be set");

        Config {
            database_url,
            logs_path,
            kafka_url,
        }
    }
}
