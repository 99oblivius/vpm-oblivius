#[derive(Debug)]
pub struct Code {
    pub code: String,
    pub active: bool,
    pub created_at: chrono::Utc,
}
