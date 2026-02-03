pub fn router() -> Router<AppState> {
    Router::new()
        .route("/license", get(license_get).patch(license_update))
}
