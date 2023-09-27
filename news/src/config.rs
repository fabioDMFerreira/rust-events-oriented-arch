#[derive(Debug, Clone)]
pub struct Config {
    pub cors_origin: String,
    pub database_url: String,
    pub logs_path: String,
    pub server_port: String,
    pub jwt_secret: String,
    pub kafka_url: String,
}

impl Config {
    pub fn init() -> Config {
        let cors_origin = std::env::var("CORS_ORIGIN").expect("KAFKA_URL must be set");
        let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let logs_path = std::env::var("LOGS_PATH").unwrap_or_else(|_| String::from(""));
        let server_port = std::env::var("PORT").unwrap_or_else(|_| String::from("8000"));
        let jwt_secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");
        let kafka_url = std::env::var("KAFKA_URL").expect("KAFKA_URL must be set");

        Config {
            cors_origin,
            database_url,
            logs_path,
            server_port,
            jwt_secret,
            kafka_url,
        }
    }
}
