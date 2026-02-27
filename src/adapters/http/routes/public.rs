use std::sync::Arc;

use askama::Template;
use askama_web::WebTemplate;
use axum::{Form, Router, extract::{State, rejection::FormRejection}, response::{IntoResponse, Redirect}, routing};
use axum_extra::extract::CookieJar;
use ::http::{HeaderMap, StatusCode, header};
use serde::Deserialize;

use crate::{
    adapters::http::{self, app_state::AppState, bounded::Bounded},
    infra::config::AppConfig,
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
    brand_name: String,
    error: Option<String>,
}

async fn landing_page(
    State(config): State<Arc<AppConfig>>,
) -> LandingTemplate {
    LandingTemplate { brand_name: config.brand_name.clone(), error: None }
}

#[derive(Debug, Clone, Deserialize)]
struct RedeemPayload {
    code: Bounded<512>,
}

async fn redeem_page(
    State(config): State<Arc<AppConfig>>,
    State(license_use_cases): State<Arc<LicenseUseCases>>,
    form: Result<Form<RedeemPayload>, FormRejection>,
) -> impl IntoResponse {
    let brand = config.brand_name.clone();
    let payload = match form {
        Ok(Form(p)) => p,
        Err(_) => return LandingTemplate { brand_name: brand, error: Some("Please enter a license or gift code.".to_string()) }.into_response(),
    };
    let token = match license_use_cases.redeem(&payload.code).await {
        Ok(t) => t,
        Err(e) => {
            let msg = match e {
                crate::app_error::AppError::InvalidLicense => "Invalid or expired license".to_string(),
                crate::app_error::AppError::ProductNotLinked => "This product was not registered. If you believe this to be an error, reach out to the seller.".to_string(),
                _ => "Something went wrong. Please try again.".to_string(),
            };
            return LandingTemplate { brand_name: brand, error: Some(msg) }.into_response();
        }
    };

    let cookie = format!(
        "redeem_result={}; HttpOnly; Secure; SameSite=Strict; Max-Age=60; Path=/",
        token
    );

    (
        StatusCode::FOUND,
        [(header::LOCATION, "/redeem".to_string()), (header::SET_COOKIE, cookie)],
    ).into_response()
}

struct ResultPackage {
    display_name: String,
    uid: String,
    latest_version: String,
}

#[derive(Template, WebTemplate)]
#[template(path = "public/result.html")]
struct ResultTemplate {
    brand_name: String,
    listing_url: String,
    packages: Vec<ResultPackage>,
}

async fn result_page(
    jar: CookieJar,
    headers: HeaderMap,
    State(config): State<Arc<AppConfig>>,
    State(package_use_cases): State<Arc<PackageUseCases>>,
) -> impl IntoResponse {
    let brand = config.brand_name.clone();
    let Some(cookie) = jar.get("redeem_result") else {
        return Redirect::to("/").into_response();
    };
    let token = cookie.value();

    let base_url = http::base_url_from_headers(&headers, &config.cors_origins);
    let listing_url = format!("{base_url}/index.json?token={token}");

    let package = match package_use_cases.get_package_for_token(token).await {
        Ok(Some(pkg)) => pkg,
        _ => return LandingTemplate { brand_name: brand, error: Some("Invalid or expired license".to_string()) }.into_response(),
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
        display_name: package.display_name,
        uid: package.uid,
        latest_version,
    }];

    ResultTemplate {
        brand_name: brand,
        listing_url,
        packages,
    }
    .into_response()
}
