use actix_files as fs;
use actix_files::NamedFile;
use actix_session::{storage::CookieSessionStore, Session, SessionMiddleware};
use actix_web::Error;
use actix_web::{cookie::Key, get, post, web, App, HttpResponse, HttpServer, Responder};
use actix_web::{HttpRequest, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use rand::{distributions::Alphanumeric, Rng};

#[derive(Serialize, Deserialize)]
struct Book {
    title: String,
    title_en: String,
    author: String,
    ganre: String,
    sentences: [String; 3],
}

async fn index(_req: HttpRequest) -> Result<NamedFile> {
    let path: PathBuf = "./static/index.html".parse().unwrap();
    Ok(NamedFile::open(path)?)
}

#[get("/api/counter")]
async fn counter(session: Session) -> Result<String, Error> {
    if let Some(count) = session.get::<i32>("counter")? {
        if count < 10 {
            session.insert("counter", count + 1)?;
        }
    } else {
        session.insert("counter", 1)?;
    }

    let book_counter = session.get::<i32>("counter")?.unwrap().to_string() + "/10";
    Ok(book_counter)
}

fn generate_placeholder(input: &str) -> String {
    let words: Vec<&str> = input.split(' ').collect();
    let mut placeholder = String::new();

    for word in words {
    let rng = rand::thread_rng();
        let random_word: String = rng.sample_iter(&Alphanumeric)
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
async fn sentences(session: Session) -> Result<String, Error> {
    let  book = Book {
        title: "Niespokojni ludzie".to_string(),
        title_en: "They both die at the end".to_string(),
        author: "Adam Silvera".to_string(),
        ganre: "fantastyka".to_string(),
        sentences: ["Przestępczy geniusz <i> IMIĘ NAZWISKO </i> otrzymuje ofertę wzbogacenia się ponad wszelkie wyobrażenie – wystarczy w tym celu wykonać zadanie, która z pozoru wydaje się niewykonalne: <br>".to_string(), "– włamać się do niesławnego <i> MIEJSCA </i> (niezdobytej wojskowej twierdzy) <br>".to_string(), "– uwolnić zakładnika (a ten może rozpętać magiczne piekło, które pochłonie cały świat).".to_string()],
    };


    let sentence2_placeholder = generate_placeholder(&book.sentences[1]);
    let sentence3_placeholder =  generate_placeholder(&book.sentences[2]);

    let state = session.get::<i32>("sentences_state")?.unwrap_or(1);

    if state < 3 {
        session.insert("sentences_state", state + 1);
    } else {
        session.insert("sentences_state", 1);
    }

    match state {
        1 => Ok(format!("<p class=\"sentences\" id=\"sen\">{} <blur hx-get=\"/api/sentences\" hx-swap=\"outerHTML\" hx-target=\"#sen\">{}</blur><blur> {}</blur></p>", book.sentences[0], sentence2_placeholder, sentence3_placeholder)),
        2 => Ok(format!("<p class=\"sentences\" id=\"sen\">{} {} <blur hx-get=\"/api/sentences\" hx-swap=\"outerHTML\" hx-target=\"#sen\">{}</blur></p>", book.sentences[0], book.sentences[1], sentence3_placeholder)),
        _ => Ok(format!("<p class=\"sentences\" id=\"sen\">{} {} {}</p>", book.sentences[0], book.sentences[1], book.sentences[2])),
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
