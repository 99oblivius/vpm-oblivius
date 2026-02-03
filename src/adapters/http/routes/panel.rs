pub fn router() -> Router<AppState> {
    Router::new()
        .route("/login", get(panel_index).post(panel_login))
        .route("/logout", post(panel_logout))
}
