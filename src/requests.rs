#[derive(Debug, Deserialize)]
pub struct NextReq {
    pub next: Option<bool>,
}

#[derive(Deserialize)]
pub struct FormData {
    pub title: String,
}

#[derive(Deserialize)]
pub struct HelpReq {
    pub number: u8,
}
