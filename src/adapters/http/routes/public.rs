pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(landing_page))
        .route("/redeem", post(redeem_key))
        .route("/index.json", get(vpm_index))
}

#[derive(Debug, Clone, Deserialize)]
struct RedeemPayload {
    code: String,
}

#[derive(Debug, Clone, Serialize)]
struct RedeemResponse {
    redirect: String,
    message: String,
    success: bool,
}
