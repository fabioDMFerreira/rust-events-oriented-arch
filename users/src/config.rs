use utils::http::middlewares::jwt_auth::JwtMiddlewareConfig;

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub jwt_expires_in: i64,
    pub jwt_max_age: i64,
    pub jwt_secret: String,
    pub kafka_url: String,
    pub server_port: String,
    pub cors_origin: String,
    pub logs_path: String,
}

impl JwtMiddlewareConfig for Config {
    fn get_jwt_secret(&self) -> String {
        return self.jwt_secret.clone();
    }
}

impl Config {
    pub fn init() -> Config {
        let cors_origin = std::env::var("CORS_ORIGIN").expect("KAFKA_URL must be set");
        let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let jwt_max_age = std::env::var("JWT_MAX_AGE").expect("JWT_MAX_AGE must be set");
        let jwt_expires_in = std::env::var("JWT_EXPIRED_IN").expect("JWT_EXPIRED_IN must be set");
        let jwt_secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");
        let kafka_url = std::env::var("KAFKA_URL").expect("KAFKA_URL must be set");
        let logs_path = std::env::var("LOGS_PATH").unwrap_or_else(|_| String::from(""));
        let server_port = std::env::var("PORT").unwrap_or_else(|_| String::from("8000"));

        Config {
            cors_origin,
            database_url,
            jwt_secret,
            jwt_expires_in: jwt_expires_in.parse::<i64>().unwrap(),
            jwt_max_age: jwt_max_age.parse::<i64>().unwrap(),
            kafka_url,
            logs_path,
            server_port,
        }
    }
}
