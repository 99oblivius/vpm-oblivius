use std::{env, time::Duration};

use dotenvy::dotenv;
use parse_duration;

pub struct AppConfig {
    pub serve_addr: String,

    pub database_url: String,
    pub packages_dir: String,

    pub jwt_secret: String,
    pub access_token_ttl: Duration,
    pub refresh_token_ttl: Duration,

    pub admin_user: String,
    pub admin_pass_hash: String,

    pub cors_origins: Vec<String>,

    pub brand_name: String,
    pub listing_id: String,
    pub listing_author: String,

    pub payhip_base_url: Option<String>,
    pub jinxxy_base_url: Option<String>,
}

impl AppConfig {
    pub fn from_env() -> Self {
        let _ = dotenv();

        let host = env::var("HOST").unwrap_or("127.0.0.1".into());
        let port: u16 = env::var("PORT")
            .unwrap_or("3000".into())
            .parse()
            .expect("PORT must be a number between 0 and 65536");
        let serve_addr = format!("{host}:{port}");

        let database_url = env::var("DATABASE_URL").unwrap_or("./data/keys.db".into());
        let packages_dir = env::var("PACKAGES_DIR").unwrap_or("./data/packages".into());

        let jwt_secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");
        if jwt_secret.len() < 32 {
            panic!("JWT_SECRET must be at least 32 bytes long");
        }

        let access_token_ttl = env::var("ACCESS_TOKEN_TTL").unwrap_or("30m".into());
        let access_token_ttl: Duration = parse_duration::parse(&access_token_ttl)
            .expect("ACCESS_TOKEN_TTL could not be turned into a Duration");

        let refresh_token_ttl = env::var("REFRESH_TOKEN_TTL").unwrap_or("1d".into());
        let refresh_token_ttl: Duration = parse_duration::parse(&refresh_token_ttl)
            .expect("REFRESH_TOKEN_TTL could not be turned into a Duration");

        let admin_user = env::var("ADMIN_USER").expect("ADMIN_USER must be set");
        let admin_pass_hash = env::var("ADMIN_PASS_HASH").expect("ADMIN_PASS_HASH must be set");

        let cors_origins: Vec<String> = env::var("CORS_ORIGINS")
            .expect("CORS_ORIGINS must be set")
            .split(',')
            .map(|s| s.trim().to_owned())
            .filter(|s| !s.is_empty())
            .collect();

        let brand_name = env::var("BRAND_NAME").unwrap_or("VPM".into());
        let listing_id = env::var("VPM_LISTING_ID").unwrap_or("com.example.vpm".into());
        let listing_author = env::var("VPM_LISTING_AUTHOR").unwrap_or("VPM".into());

        let payhip_base_url = env::var("PAYHIP_BASE_URL").ok();
        let jinxxy_base_url = env::var("JINXXY_BASE_URL").ok();

        Self {
            serve_addr,

            database_url,
            packages_dir,

            jwt_secret,
            access_token_ttl,
            refresh_token_ttl,

            admin_user,
            admin_pass_hash,

            cors_origins,

            brand_name,
            listing_id,
            listing_author,

            payhip_base_url,
            jinxxy_base_url,
        }
    }
}
