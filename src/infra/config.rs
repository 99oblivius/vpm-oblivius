use dotenv::dotenv;
use parse_duration;
use std::{env, time::Duration};

pub struct AppConfig {
    pub serve_addr: String,

    pub database_url: String,
    pub packages_dir: String,

    pub jwt_secret: String,
    pub access_token_ttl: Duration,
    pub refresh_token_ttl: Duration,

    pub cors_origins: Vec<String>,

    pub payhip_api_url: Option<String>,
    pub payhip_api_key: Option<String>,
}

impl AppConfig {
    pub fn from_env() -> Self {
        dotenv();

        let host = env::var("HOST").unwrap_or("127.0.0.1".into());
        let port: u16 = env::var("PORT")
            .unwrap_or("3000".into())
            .parse()
            .expect("PORT must be a number between 0 and 65536");
        let serve_addr = format!("{host}:{port}");

        let database_url = env::var("DATABASE_URL").unwrap_or("./data/keys.db".into());
        let packages_dir = env::var("PACKAGES_DIR").unwrap_or("./data/packages".into());

        let jwt_secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");

        let access_token_ttl = env::var("ACCESS_TOKEN_TTL").unwrap_or("1d".into());
        let access_token_ttl: Duration = parse_duration::parse(&access_token_ttl)
            .expect("ACCESS_TOKEN_TTL could not be turned into a Duration");

        let refresh_token_ttl = env::var("REFRESH_TOKEN_TTL").unwrap_or("1h".into());
        let refresh_token_ttl: Duration = parse_duration::parse(&refresh_token_ttl)
            .expect("REFRESH_TOKEN_TTL could not be turned into a Duration");

        let cors_origins: Vec<String> = env::var("CORS_ORIGINS")
            .expect("CORS_ORIGINS must be set")
            .split(',')
            .map(|s| s.trim().to_owned())
            .filter(|s| !s.is_empty())
            .collect();

        let payhip_api_url = env::var("PAYHIP_API_URL").ok();
        let payhip_api_key = env::var("PAYHIP_API_KEY").ok();

        Self {
            serve_addr,

            database_url,
            packages_dir,

            jwt_secret,
            access_token_ttl,
            refresh_token_ttl,

            cors_origins,

            payhip_api_url,
            payhip_api_key,
        }
    }
}
