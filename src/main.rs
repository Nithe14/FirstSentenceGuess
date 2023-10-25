mod book;
mod requests;

use actix_files as fs;
use actix_files::NamedFile;
use actix_session::{storage::CookieSessionStore, Session, SessionMiddleware};
use actix_web::{
    cookie::Key, get, post, web, App, Error, HttpRequest, HttpResponse, HttpServer, Responder,
    Result,
};
use book::Book;
use rand::{distributions::Alphanumeric, Rng};
use requests::GetReq;
use serde_json;
use std::fs::File;
use std::io::{Error as IoError, Read};
use std::path::PathBuf;

#[macro_use]
extern crate serde;

async fn index(_req: HttpRequest) -> Result<NamedFile> {
    let path: PathBuf = "./static/index.html".parse().unwrap();
    Ok(NamedFile::open(path)?)
}

#[get("/api/counter")]
async fn counter(session: Session) -> Result<String, Error> {
    if let Some(count) = session.get::<usize>("counter")? {
        if count < 10 {
            let _ = session.insert("counter", count + 1)?;
        }
    } else {
        let _ = session.insert("counter", 0)?;
    }

    let book_counter = session.get::<usize>("counter")?.unwrap() + 1;
    Ok(book_counter.to_string() + "/10")
}

fn read_db(n: usize) -> Result<Book, Box<dyn std::error::Error>> {
    let mut file = File::open("db.json")?;
    let mut data = String::new();
    file.read_to_string(&mut data)?;
    let books: Vec<Book> = serde_json::from_str(&data)?;
    if n > books.len() {
        return Err(Box::new(IoError::new(
            std::io::ErrorKind::Other,
            "Out of index! Book not found.",
        )));
    }
    Ok(books[n].clone())
}

fn generate_placeholder(input: &str) -> String {
    let words: Vec<&str> = input.split(' ').collect();
    let mut placeholder = String::new();

    for word in words {
        let rng = rand::thread_rng();
        let random_word: String = rng
            .sample_iter(&Alphanumeric)
            .take(word.len())
            .map(char::from)
            .collect();
        if word == "<br>" || word == "</br>" {
            placeholder.push_str(word);
        } else {
            placeholder.push_str(&random_word);
        }
        placeholder.push(' ');
    }
    //delete additional space
    placeholder.pop();

    placeholder
}

#[get("/api/sentences")]
async fn sentences(session: Session, params: web::Query<GetReq>) -> Result<String, Error> {
    let count = session.get::<usize>("counter")?.unwrap_or(0);
    let db_response = read_db(count);
    let book: Book;
    match db_response {
        Ok(value) => {
            book = value;
        }
        Err(error) => {
            eprint!("ERROR: {:?}", error);
            return Ok(
                "<p class=\"sentences\" id=\"sen\">Oops! Something went wrong.</p>".to_string(),
            );
        }
    }

    let sentence2_placeholder = generate_placeholder(&book.sentences[1]);
    let sentence3_placeholder = generate_placeholder(&book.sentences[2]);

    let state = session.get::<i32>("sentences_state")?.unwrap_or(1);

    if params.next.unwrap_or(false) == true && state < 3 {
        let _ = session.insert("sentences_state", state + 1);
    } else if params.next.unwrap_or(false) == true {
        let _ = session.insert("sentences_state", 1);
    }
    match session.get::<i32>("sentences_state")?.unwrap_or(1) {
        2 => Ok(format!("<p class=\"sentences\" id=\"sen\">{} {} <blur hx-get=\"/api/sentences?next=true\" hx-swap=\"outerHTML\" hx-target=\"#sen\">{}</blur></p>", book.sentences[0], book.sentences[1], sentence3_placeholder)),
        3 => Ok(format!("<p class=\"sentences\" id=\"sen\">{} {} {}</p>", book.sentences[0], book.sentences[1], book.sentences[2])),
        _ => Ok(format!("<p class=\"sentences\" id=\"sen\">{} <blur hx-get=\"/api/sentences?next=true\" hx-swap=\"outerHTML\" hx-target=\"#sen\">{}</blur><blur> {}</blur></p>", book.sentences[0], sentence2_placeholder, sentence3_placeholder)),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .wrap(
                // create cookie based session middleware
                SessionMiddleware::builder(CookieSessionStore::default(), Key::from(&[0; 64]))
                    .cookie_secure(false)
                    .build(),
            )
            .service(counter)
            .service(sentences)
            .service(fs::Files::new("/static", "static"))
            .service(fs::Files::new("/static/svg", "static/css"))
            .route("/", web::get().to(index))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
