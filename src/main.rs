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
use requests::{FormData, GetReq};
use serde_json;
use std::fs::File;
use std::io::{Error as IoError, Read};
use std::path::PathBuf;
use tera::{Context, Tera};

#[macro_use]
extern crate serde;

async fn index(_req: HttpRequest) -> Result<NamedFile> {
    let path: PathBuf = "./static/index.html".parse().unwrap();
    Ok(NamedFile::open(path)?)
}

fn db_len() -> Result<usize, Box<dyn std::error::Error>> {
    let mut file = File::open("db.json")?;
    let mut data = String::new();
    file.read_to_string(&mut data)?;
    let books: Vec<Book> = serde_json::from_str(&data)?;

    Ok(books.len())
}

fn read_db(n: usize) -> Result<Book, Box<dyn std::error::Error>> {
    let mut file = File::open("db.json")?;
    let mut data = String::new();
    file.read_to_string(&mut data)?;
    let books: Vec<Book> = serde_json::from_str(&data)?;
    if n >= books.len() {
        return Err(Box::new(IoError::new(
            std::io::ErrorKind::Other,
            format!("Out of index [{}]! Book not found.", n),
        )));
    }
    Ok(books[n].clone())
}

fn get_template(template: &str, context: Context) -> Result<String, Box<dyn std::error::Error>> {
    let mut tera = match Tera::parse("templates/pl/*") {
        Ok(t) => t,
        Err(e) => {
            eprint!("Parsing error(s): {}", e);
            return Ok("Oops! Something went wrong.".to_string());
        }
    };
    tera.build_inheritance_chains()?;
    Ok(tera.render(template, &context).unwrap())
}

#[get("/api/counter")]
async fn counter(session: Session) -> Result<HttpResponse, Error> {
    let db_size = db_len()?;
    if let Some(count) = session.get::<usize>("counter")? {
        if count < db_size - 1 {
            session.insert("counter", count + 1)?;
        }
    } else {
        session.insert("counter", 0)?;
    }

    let book_counter = session.get::<usize>("counter")?.unwrap();
    Ok(HttpResponse::Ok()
        .insert_header(("HX-Trigger", "newBook"))
        .body(format!("{}/10", book_counter + 1)))
}

#[get("/api/new-book")]
async fn new_book(session: Session) -> Result<HttpResponse, Error> {
    let _ = session.insert("sentences_state", 1);
    let render = get_template("index.html", Context::new());
    Ok(HttpResponse::Ok().body(render?))
}

#[post("/api/check-book")]
async fn check_book(session: Session, form: web::Form<FormData>) -> Result<HttpResponse, Error> {
    let count = session.get::<usize>("counter")?.unwrap_or(0);
    let db_response = read_db(count);
    let book: Book;
    let mut context = Context::new();
    let render: Result<_, _>;
    match db_response {
        Ok(value) => {
            book = value;
        }
        Err(error) => {
            book = Book::empty();
            eprint!("ERROR: {:?}", error);
        }
    }
    if book.title.to_lowercase() == form.title.to_lowercase() {
        context.insert("title", &book.title);
        context.insert("author", &book.author);
        render = get_template("correct.html", context);
        return Ok(HttpResponse::Ok().body(render?));
    }

    render = get_template("wrong.html", context);
    Ok(HttpResponse::Ok()
        .insert_header(("HX-Retarget", "#field"))
        .insert_header(("HX-Reswap", "outerHTML"))
        .body(render?))
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
    let mut context = Context::new();
    match db_response {
        Ok(value) => {
            book = value;
        }
        Err(error) => {
            eprint!("ERROR: {:?}", error);
            return Ok("Oops! Something went wrong.".to_string());
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
        2 => {
            context.insert("sentence1", &book.sentences[0]);
            context.insert("sentence2", &book.sentences[1]);
            context.insert("sentence3", &sentence3_placeholder);
            return Ok(get_template("sentence2.html", context)?);
        }
        3 => {
            context.insert("sentence1", &book.sentences[0]);
            context.insert("sentence2", &book.sentences[1]);
            context.insert("sentence3", &book.sentences[2]);
            return Ok(get_template("sentence3.html", context)?);
        }
        _ => {
            context.insert("sentence1", &book.sentences[0]);
            context.insert("sentence2", &sentence2_placeholder);
            context.insert("sentence3", &sentence3_placeholder);
            return Ok(get_template("sentence1.html", context)?);
        }
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
            .service(check_book)
            .service(new_book)
            .service(fs::Files::new("/static", "static"))
            .service(fs::Files::new("/static/svg", "static/css"))
            .route("/", web::get().to(index))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
