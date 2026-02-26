use std::sync::Arc;

use askama::Template;
use askama_web::WebTemplate;
use axum::{Form, Router, extract::State, response::{IntoResponse, Redirect}, routing};
use axum_extra::extract::CookieJar;
use http::{HeaderMap, StatusCode, header};
use serde::Deserialize;

use crate::{
    adapters::http::{app_state::AppState, bounded::Bounded},
    app_error::AppResult,
    use_cases::{license::LicenseUseCases, packages::PackageUseCases},
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", routing::get(landing_page))
        .route("/redeem", routing::post(redeem_page).get(result_page))
        .layer(crate::adapters::http::rate_limit::per_ip(10, 20))
}

#[derive(Template, WebTemplate)]
#[template(path = "public/landing.html")]
struct LandingTemplate {
    error: Option<String>,
}

async fn landing_page() -> LandingTemplate {
    LandingTemplate { error: None }
}

#[derive(Debug, Clone, Deserialize)]
struct RedeemPayload {
    code: Bounded<512>,
}

async fn redeem_page(
    State(license_use_cases): State<Arc<LicenseUseCases>>,
    Form(payload): Form<RedeemPayload>,
) -> AppResult<impl IntoResponse> {
    let token = license_use_cases.redeem(&payload.code).await?;

    let cookie = format!(
        "redeem_result={}; HttpOnly; Secure; SameSite=Strict; Max-Age=60; Path=/",
        token
    );

    Ok((
        StatusCode::FOUND,
        [(header::LOCATION, "/redeem".to_string()), (header::SET_COOKIE, cookie)],
    ))
}

struct ResultPackage {
    name: String,
    uid: String,
    latest_version: String,
}

#[derive(Template, WebTemplate)]
#[template(path = "public/result.html")]
struct ResultTemplate {
    listing_url: String,
    packages: Vec<ResultPackage>,
}

async fn result_page(
    jar: CookieJar,
    headers: HeaderMap,
    State(package_use_cases): State<Arc<PackageUseCases>>,
) -> impl IntoResponse {
    let Some(cookie) = jar.get("redeem_result") else {
        return Redirect::to("/").into_response();
    };
    let token = cookie.value();

    let host = headers
        .get(header::HOST)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("localhost");
    let scheme = if host.starts_with("localhost") || host.starts_with("127.0.0.1") {
        "http"
    } else {
        "https"
    };
    let listing_url = format!("{scheme}://{host}/index.json?token={token}");

    let package = match package_use_cases.get_package_for_token(token).await {
        Ok(Some(pkg)) => pkg,
        _ => return Redirect::to("/").into_response(),
    };

    let versions = package_use_cases
        .get_versions(&package.uid)
        .await
        .unwrap_or_default();

    let latest_version = versions
        .first()
        .map(|v| v.version.clone())
        .unwrap_or_else(|| "-".to_string());

    let packages = vec![ResultPackage {
        name: package.name,
        uid: package.uid,
        latest_version,
    }];

    ResultTemplate {
        listing_url,
        packages,
    }
    .into_response()
}
