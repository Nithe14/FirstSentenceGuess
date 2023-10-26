#[derive(Debug, Deserialize)]
pub struct GetReq {
    pub next: Option<bool>,
}

#[derive(Deserialize)]
pub struct FormData {
    pub title: String,
}
