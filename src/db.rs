use crate::book::Book;
use serde_json;
use std::fs::File;
use std::io::BufReader;

pub fn load_data_from_file(file_path: &str) -> Result<Vec<Book>, Box<dyn std::error::Error>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let data: Vec<Book> = serde_json::from_reader(reader)?;
    Ok(data)
}

pub async fn get_entry(data: &Vec<Book>, n: usize) -> Result<Book, Box<dyn std::error::Error>> {
    if let Some(book) = data.get(n) {
        Ok(book.clone())
    } else {
        Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Out of index [{}]! Book not found.", n),
        )))
    }
}
