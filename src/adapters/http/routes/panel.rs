use std::sync::Arc;

use argon2::{Argon2, PasswordHash, PasswordVerifier};
use askama::Template;
use askama_web::WebTemplate;
use axum::{
    Form, Router,
    extract::{Path, Query, State, rejection::FormRejection},
    http::{StatusCode, header},
    middleware::{self, Next},
    response::{IntoResponse, Redirect, Response},
    routing,
};
use axum_extra::extract::CookieJar;
use serde::Deserialize;
use sha2::{Sha256, Digest};
use subtle::ConstantTimeEq;
use crate::adapters::http::rate_limit;

use crate::{
    adapters::{
        http::app_state::{AppState, AuthState},
        utils::jwt::{self, TokenType},
    },
    infra::config::AppConfig,
};

pub fn router(state: AppState) -> Router<AppState> {
    let protected = Router::new()
        .route("/", routing::get(|| async { Redirect::to("/panel/packages") }))
        .route("/packages", routing::get(panel_packages))
        .route("/packages/{uid}", routing::get(panel_package_detail))
        .route("/licenses", routing::get(panel_licenses))
        .route("/markets", routing::get(panel_markets))
        .layer(middleware::from_fn_with_state(state, panel_auth));

    Router::new()
        .route("/login", routing::get(panel_login_page))
        .route(
            "/login",
            routing::post(panel_login).layer(rate_limit::per_ip(2, 5)),
        )
        .route("/logout", routing::post(panel_logout))
        .route("/refresh", routing::post(panel_refresh))
        .merge(protected)
}

async fn panel_auth(
    State(config): State<Arc<AppConfig>>,
    State(auth): State<AuthState>,
    jar: CookieJar,
    request: axum::extract::Request,
    next: Next,
) -> Response {
    let Some(token) = jar.get("admin_token").map(|c| c.value().to_string()) else {
        return Redirect::to("/panel/login").into_response();
    };

    if jwt::validate_access_token(&config.jwt_secret, &token, auth.issued_after()).is_err() {
        return Redirect::to("/panel/login").into_response();
    }

    next.run(request).await
}

#[derive(Template, WebTemplate)]
#[template(path = "panel/login.html")]
struct LoginTemplate {
    error: Option<String>,
}

async fn panel_login_page() -> LoginTemplate {
    LoginTemplate { error: None }
}

#[derive(Debug, Deserialize)]
struct LoginPayload {
    username: String,
    password: String,
}

fn verify_password(password: &str, expected_hash: &str) -> bool {
    let Ok(parsed_hash) = PasswordHash::new(expected_hash) else {
        return false;
    };
    Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok()
}

/// Constant-time username comparison using SHA-256 normalization + subtle::ct_eq.
fn verify_username(input: &str, expected: &str) -> bool {
    let hash_input = Sha256::digest(input.as_bytes());
    let hash_expected = Sha256::digest(expected.as_bytes());
    hash_input.ct_eq(&hash_expected).into()
}

async fn panel_login(
    State(config): State<Arc<AppConfig>>,
    form: Result<Form<LoginPayload>, FormRejection>,
) -> impl IntoResponse {
    let payload = match form {
        Ok(Form(p)) => p,
        Err(_) => return LoginTemplate { error: Some("Invalid credentials".to_string()) }.into_response(),
    };
    if payload.username.len() > 256 || payload.password.len() > 1024 {
        return LoginTemplate {
            error: Some("Invalid credentials".to_string()),
        }
        .into_response();
    }

    let user_ok = verify_username(&payload.username, &config.admin_user);
    let pass_ok = verify_password(&payload.password, &config.admin_pass_hash);

    if !user_ok || !pass_ok {
        return LoginTemplate {
            error: Some("Invalid credentials".to_string()),
        }
        .into_response();
    }

    let access_token = match jwt::create_token(
        &config.jwt_secret,
        config.access_token_ttl,
        &config.admin_user,
        TokenType::Access,
    ) {
        Ok(t) => t,
        Err(_) => {
            return LoginTemplate {
                error: Some("Internal error".to_string()),
            }
            .into_response()
        }
    };

    let refresh_token = match jwt::create_token(
        &config.jwt_secret,
        config.refresh_token_ttl,
        &config.admin_user,
        TokenType::Refresh,
    ) {
        Ok(t) => t,
        Err(_) => {
            return LoginTemplate {
                error: Some("Internal error".to_string()),
            }
            .into_response()
        }
    };

    let access_cookie = format!(
        "admin_token={}; HttpOnly; Secure; SameSite=Strict; Max-Age={}; Path=/",
        access_token,
        config.access_token_ttl.as_secs()
    );
    let refresh_cookie = format!(
        "admin_refresh={}; HttpOnly; Secure; SameSite=Strict; Max-Age={}; Path=/panel/refresh",
        refresh_token,
        config.refresh_token_ttl.as_secs()
    );

    let mut headers = header::HeaderMap::new();
    headers.insert(header::LOCATION, "/panel/packages".parse().unwrap());
    headers.append(header::SET_COOKIE, access_cookie.parse().unwrap());
    headers.append(header::SET_COOKIE, refresh_cookie.parse().unwrap());

    (StatusCode::FOUND, headers).into_response()
}

async fn panel_logout(State(auth): State<AuthState>) -> impl IntoResponse {
    auth.revoke_all();

    let clear_access = "admin_token=; HttpOnly; Secure; SameSite=Strict; Max-Age=0; Path=/";
    let clear_refresh =
        "admin_refresh=; HttpOnly; Secure; SameSite=Strict; Max-Age=0; Path=/panel/refresh";

    let mut headers = header::HeaderMap::new();
    headers.insert(header::LOCATION, "/panel/login".parse().unwrap());
    headers.append(header::SET_COOKIE, clear_access.parse().unwrap());
    headers.append(header::SET_COOKIE, clear_refresh.parse().unwrap());

    (StatusCode::FOUND, headers)
}

async fn panel_refresh(
    State(config): State<Arc<AppConfig>>,
    State(auth): State<AuthState>,
    jar: CookieJar,
) -> impl IntoResponse {
    let Some(refresh) = jar.get("admin_refresh").map(|c| c.value().to_string()) else {
        return StatusCode::UNAUTHORIZED.into_response();
    };

    let claims =
        match jwt::validate_refresh_token(&config.jwt_secret, &refresh, auth.issued_after()) {
            Ok(c) => c,
            Err(_) => return StatusCode::UNAUTHORIZED.into_response(),
        };

    let access_token = match jwt::create_token(
        &config.jwt_secret,
        config.access_token_ttl,
        &claims.sub,
        TokenType::Access,
    ) {
        Ok(t) => t,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };

    let refresh_token = match jwt::create_token(
        &config.jwt_secret,
        config.refresh_token_ttl,
        &claims.sub,
        TokenType::Refresh,
    ) {
        Ok(t) => t,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };

    let access_cookie = format!(
        "admin_token={}; HttpOnly; Secure; SameSite=Strict; Max-Age={}; Path=/",
        access_token,
        config.access_token_ttl.as_secs()
    );
    let refresh_cookie = format!(
        "admin_refresh={}; HttpOnly; Secure; SameSite=Strict; Max-Age={}; Path=/panel/refresh",
        refresh_token,
        config.refresh_token_ttl.as_secs()
    );

    let mut headers = header::HeaderMap::new();
    headers.append(header::SET_COOKIE, access_cookie.parse().unwrap());
    headers.append(header::SET_COOKIE, refresh_cookie.parse().unwrap());

    (StatusCode::OK, headers).into_response()
}

#[derive(Template, WebTemplate)]
#[template(path = "panel/packages.html")]
struct PackagesTemplate {
    packages: Vec<crate::domain::Package>,
}

async fn panel_packages(
    State(package_use_cases): State<Arc<crate::use_cases::packages::PackageUseCases>>,
) -> impl IntoResponse {
    let packages = package_use_cases.list().await.unwrap_or_default();
    PackagesTemplate { packages }
}

struct MarketLink {
    market: String,
    product_id: String,
}

#[derive(Template, WebTemplate)]
#[template(path = "panel/package_detail.html")]
struct PackageDetailTemplate {
    package: crate::domain::Package,
    versions: Vec<crate::domain::PackageVersion>,
    markets: Vec<String>,
    linked_markets: Vec<MarketLink>,
}

async fn panel_package_detail(
    State(package_use_cases): State<Arc<crate::use_cases::packages::PackageUseCases>>,
    State(store): State<Arc<crate::domain::MarketCredentialStore>>,
    Path(uid): Path<String>,
) -> Response {
    let Ok(Some(package)) = package_use_cases.get_by_uid(&uid).await else {
        return Redirect::to("/panel/packages").into_response();
    };
    let versions = package_use_cases.get_versions(&uid).await.unwrap_or_default();
    let markets: Vec<String> = store.list().into_iter().map(|c| c.market).collect();
    let linked_markets = package_use_cases
        .get_market_links(&uid)
        .await
        .unwrap_or_default()
        .into_iter()
        .map(|(market, product_id)| MarketLink { market, product_id })
        .collect();
    PackageDetailTemplate { package, versions, markets, linked_markets }.into_response()
}

#[derive(Debug, Deserialize)]
struct LicensesQuery {
    #[serde(default = "default_cursor")]
    cursor: i64,
    #[serde(default = "default_page_size")]
    page_size: i64,
}

fn default_cursor() -> i64 { i64::MAX }
fn default_page_size() -> i64 { 50 }

struct PackageOption {
    uid: String,
    display_name: String,
}

#[derive(Template, WebTemplate)]
#[template(path = "panel/licenses.html")]
struct LicensesTemplate {
    licenses: Vec<crate::domain::License>,
    packages: Vec<PackageOption>,
    page_size: i64,
    show_first_page: bool,
    next_cursor: Option<i64>,
}

async fn panel_licenses(
    State(license_use_cases): State<Arc<crate::use_cases::license::LicenseUseCases>>,
    State(package_use_cases): State<Arc<crate::use_cases::packages::PackageUseCases>>,
    Query(query): Query<LicensesQuery>,
) -> impl IntoResponse {
    let page_size = query.page_size.clamp(1, 1000);
    let licenses = license_use_cases
        .list(&query.cursor, &page_size)
        .await
        .unwrap_or_default();
    let next_cursor = if licenses.len() as i64 == query.page_size {
        licenses.last().map(|l| l.id)
    } else {
        None
    };
    let packages = package_use_cases
        .list()
        .await
        .unwrap_or_default()
        .into_iter()
        .map(|p| PackageOption { uid: p.uid, display_name: p.display_name })
        .collect();
    let show_first_page = query.cursor != i64::MAX;
    LicensesTemplate {
        licenses,
        packages,
        page_size: query.page_size,
        show_first_page,
        next_cursor,
    }
}

struct MarketView {
    market: String,
    base_url: String,
    active: bool,
    updated_at: String,
}

#[derive(Template, WebTemplate)]
#[template(path = "panel/markets.html")]
struct MarketsTemplate {
    markets: Vec<MarketView>,
}

async fn panel_markets(
    State(store): State<Arc<crate::domain::MarketCredentialStore>>,
) -> impl IntoResponse {
    let markets = store
        .list()
        .into_iter()
        .map(|c| MarketView {
            market: c.market,
            base_url: c.base_url,
            active: c.active,
            updated_at: c.updated_at,
        })
        .collect();
    MarketsTemplate { markets }
}
