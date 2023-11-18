#[derive(Serialize, Deserialize, Clone)]
pub struct Book {
    pub title: String,
    pub title_alter: String,
    pub author: String,
    pub ganre: String,
    pub sentences: [String; 3],
}

impl Book {
    pub fn empty() -> Self {
        Book {
            title: String::new(),
            title_alter: String::new(),
            author: String::new(),
            ganre: String::new(),
            sentences: [String::new(), String::new(), String::new()],
        }
    }
}
