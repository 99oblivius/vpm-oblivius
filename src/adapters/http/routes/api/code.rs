pub fn router() -> Router<AppState> {
    Router::new()
        .route("/code", post(code_create).patch(code_update).delete(code_delete))
}
