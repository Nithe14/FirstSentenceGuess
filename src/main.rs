mod book;
mod requests;

use actix_files as fs;
use actix_files::NamedFile;
use actix_session::{storage::CookieSessionStore, Session, SessionMiddleware};
use actix_web::{
    cookie::Key, get, post, web, App, Error, HttpRequest, HttpResponse, HttpServer, Result,
};
use book::Book;
use rand::{distributions::Alphanumeric, Rng};
use requests::{FormData, HelpReq, NextReq};
use serde_json;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::io::Error as IoError;
use std::path::PathBuf;
use tera::{Context, Tera};

#[macro_use]
extern crate serde;

fn load_data_from_file(file_path: &str) -> Result<Vec<Book>, Box<dyn std::error::Error>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let data: Vec<Book> = serde_json::from_reader(reader)?;
    Ok(data)
}

async fn get_entry(
    data: &web::Data<Vec<Book>>,
    n: usize,
) -> Result<Book, Box<dyn std::error::Error>> {
    let books = &data;
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

#[get("/")]
async fn index(_req: HttpRequest) -> Result<NamedFile> {
    let path: PathBuf = "./static/index.html".parse().unwrap();
    Ok(NamedFile::open(path)?)
}

#[get("/api/counter")]
async fn counter(data: web::Data<Vec<Book>>, session: Session) -> Result<HttpResponse, Error> {
    let db_size = data.len();
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

#[get("/api/index")]
async fn render_index(
    data: web::Data<Vec<Book>>,
    session: Session,
    params: web::Query<NextReq>,
) -> Result<HttpResponse, Error> {
    let mut context = Context::new();
    let count = session.get::<usize>("counter")?.unwrap_or(0);

    if count + 1 >= data.len() && params.next.unwrap_or(false)
        || session
            .get(&format!("book_{}_done", &data.len() - 1))?
            .unwrap_or(false)
    {
        //let mut books_points = Vec::new();
        let mut books = HashMap::new();
        let all_points = session.get::<f32>("all_points")?.unwrap_or(0.00);
        let progress = (all_points / (data.len() as f32 * 5.00)) * 100.00;
        context.insert("progress", &progress);
        context.insert("all_points", &all_points);
        for book_number in 0..data.len() {
            let points = session
                .get::<f32>(format!("points_{}", book_number).as_str())?
                .unwrap_or(0.00);
            //books_points.push(points);
            let book = get_entry(&data, book_number).await?;
            let key = format!("{} \"{}\"", book.author, book.title);
            books.insert(key, points);
        }
        context.insert("books", &books);
        let render = get_template("finish.html", context);
        return Ok(HttpResponse::Ok().body(render?));
    }
    let db_size = data.len();
    if let Some(count) = session.get::<usize>("counter")? {
        if count < db_size && params.next.unwrap_or(false) {
            session.insert("counter", count + 1)?;
            session.insert("current_points", 5)?;
        }
    } else {
        session.insert("counter", 0)?;
        session.insert("current_points", 5)?;
    }

    //help states
    let help1_state = session.get::<bool>("help1_state")?.unwrap_or(true);
    let help2_state = session.get::<bool>("help2_state")?.unwrap_or(true);

    match (help1_state, help2_state) {
        (false, false) => {
            context.insert("help1_state", &"");
            context.insert("help2_state", &"");
        }
        (true, false) => {
            context.insert(
                "help1_state",
                &get_template("help1-avail.html", Context::new())?,
            );
            session.insert("help1_state", true)?;
            context.insert("help2_state", &"");
        }
        (false, true) => {
            context.insert("help1_state", &"");
            context.insert(
                "help2_state",
                &get_template("help2-avail.html", Context::new())?,
            );
            session.insert("help2_state", true)?;
        }
        (_, _) => {
            context.insert(
                "help1_state",
                &get_template("help1-avail.html", Context::new())?,
            );
            session.insert("help1_state", true)?;
            context.insert(
                "help2_state",
                &get_template("help2-avail.html", Context::new())?,
            );
            session.insert("help2_state", true)?;
        }
    }

    let _ = session.insert("sentences_state", 1);
    let all_points = session.get::<f32>("all_points")?.unwrap_or(0.00);
    context.insert("all_points", &all_points);
    let progress = (all_points / (data.len() as f32 * 5.00)) * 100.00;
    context.insert("progress", &progress);
    context.insert("counter", &session.get::<usize>("counter")?.unwrap_or(0));
    let render = get_template("index.html", context);
    Ok(HttpResponse::Ok().body(render?))
}

#[post("/api/give-up")]
async fn give_up(data: web::Data<Vec<Book>>, session: Session) -> Result<HttpResponse, Error> {
    let count = session.get::<usize>("counter")?.unwrap_or(0);
    session.insert(format!("points_{}", count.to_string()), 0)?;
    session.insert(format!("book_{}_done", count), true)?;
    let all_points = session.get::<f32>("all_points")?.unwrap_or(0.00); //no points adding
    let book: Book = match get_entry(&data, count).await {
        Ok(value) => value,
        Err(error) => {
            eprint!("ERROR: {:?}", error);
            Book::empty()
        }
    };

    let mut context = Context::new();
    let render: Result<_, _>;

    context.insert("title", &book.title);
    context.insert("author", &book.author);
    let progress = (all_points / (data.len() as f32 * 5.00)) * 100.00;
    context.insert("progress", &progress);
    context.insert("all_points", &all_points);
    context.insert("counter", &count);
    render = get_template("give-up.html", context);
    Ok(HttpResponse::Ok().body(render?))
}

#[post("/api/check-book")]
async fn check_book(
    data: web::Data<Vec<Book>>,
    session: Session,
    form: web::Form<FormData>,
) -> Result<HttpResponse, Error> {
    let count = session.get::<usize>("counter")?.unwrap_or(0);
    let current_points = session.get::<f32>("current_points")?.unwrap_or(0.00);
    let mut all_points = session.get::<f32>("all_points")?.unwrap_or(0.00);
    let book: Book = match get_entry(&data, count).await {
        Ok(value) => value,
        Err(error) => {
            eprint!("ERROR: {:?}", error);
            Book::empty()
        }
    };

    let mut context = Context::new();
    let render: Result<_, _>;

    context.insert("guess", &form.title);
    if book.title.to_lowercase() == form.title.to_lowercase()
        || book.title_alter.to_lowercase() == form.title.to_lowercase()
    {
        let is_done: bool = session
            .get::<bool>(format!("book_{}_done", count).as_str())?
            .unwrap_or(false);
        if is_done == false {
            session.insert(format!("points_{}", count.to_string()), current_points)?;
            session.insert(format!("book_{}_done", count), true)?;
            all_points = all_points + current_points;
            session.insert("all_points", all_points)?;
        }
        context.insert("all_points", &all_points);
        context.insert("title", &book.title);
        context.insert("author", &book.author);
        context.insert("counter", &count);
        let progress = (all_points / (data.len() as f32 * 5.00)) * 100.00;
        context.insert("progress", &progress);
        render = get_template("correct.html", context);
        return Ok(HttpResponse::Ok().body(render?));
    }

    if current_points > 1.00 {
        session.insert("current_points", (current_points - 1.00) as u8)?;
    }
    render = get_template("wrong.html", context);
    Ok(HttpResponse::Ok()
        .insert_header(("HX-Retarget", "#frm"))
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
async fn sentences(
    data: web::Data<Vec<Book>>,
    session: Session,
    params: web::Query<NextReq>,
) -> Result<String, Error> {
    let count = session.get::<usize>("counter")?.unwrap_or(0);
    let mut current_points = session.get::<i8>("current_points")?.unwrap_or(0);
    let db_response = get_entry(&data, count).await;
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
        if current_points > 1 {
            current_points = current_points - 1;
        }
    } else if params.next.unwrap_or(false) == true {
        let _ = session.insert("sentences_state", 1);
    }
    match session.get::<i32>("sentences_state")?.unwrap_or(1) {
        2 => {
            session.insert("current_points", current_points)?;
            context.insert("sentence1", &book.sentences[0]);
            context.insert("sentence2", &book.sentences[1]);
            context.insert("sentence3", &sentence3_placeholder);
            return Ok(get_template("sentence2.html", context)?);
        }
        3 => {
            session.insert("current_points", current_points)?;
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

#[get("/api/get-help")]
async fn get_help(
    data: web::Data<Vec<Book>>,
    session: Session,
    params: web::Query<HelpReq>,
) -> Result<String, Error> {
    let count = session.get::<usize>("counter")?.unwrap_or(0);
    let current_points = session.get::<u8>("current_points")?.unwrap_or(0);
    let db_response = get_entry(&data, count).await;
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
    if current_points > 1 {
        session.insert("current_points", current_points - 1)?;
    }
    match params.number {
        1 => {
            session.insert("help1_state", false)?;
            context.insert("help1", &book.ganre);
            Ok(get_template("help1.html", context)?)
        }
        _ => {
            session.insert("help2_state", false)?;
            context.insert("help2", &book.author);
            Ok(get_template("help2.html", context)?)
        }
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let data =
        web::Data::new(load_data_from_file("db.json").expect("Failed to read database json file!"));
    HttpServer::new(move || {
        App::new()
            .wrap(
                // create cookie based session middleware
                SessionMiddleware::builder(CookieSessionStore::default(), Key::from(&[0; 64]))
                    .cookie_secure(false)
                    .build(),
            )
            .app_data(data.clone())
            .service(counter)
            .service(sentences)
            .service(give_up)
            .service(get_help)
            .service(check_book)
            .service(render_index)
            .service(index)
            .service(fs::Files::new("/static", "static"))
            .service(fs::Files::new("/static/svg", "static/css"))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
