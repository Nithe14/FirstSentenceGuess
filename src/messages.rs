#[derive(Serialize, Deserialize)]
pub struct Messages {
    pub correct: String,
    pub results: String,
    pub next_button_msg: String,
    pub give_up_page_msg: String,
    pub give_up_button_msg: String,
    pub check_button_msg: String,
    pub input_placeholder: String,
    pub help1_button_msg: String,
    pub help2_button_msg: String,
}
pub fn parse_messages(lang: &String) -> Result<Messages, std::io::Error> {
    let messages_path = std::env::current_dir()?.join(format!("templates/{}/messages.toml", lang));
    let messages_string = std::fs::read_to_string(&messages_path).map_err(|e| {
        eprintln!("Failed to parse messages file: {:?} {}", messages_path, e);
        std::io::Error::new(std::io::ErrorKind::NotFound, "500 Internal Server Error")
    })?;
    toml::from_str::<Messages>(&messages_string).map_err(|e| {
        eprintln!("Failed to parse messages file: {}", e);
        std::io::Error::new(std::io::ErrorKind::InvalidData, "500 Internal Server Error")
    })
}
