use actix_files as fs;
use actix_files::NamedFile;
use actix_session::{storage::CookieSessionStore, Session, SessionMiddleware};
use actix_web::Error;
use actix_web::{cookie::Key, get, post, web, App, HttpResponse, HttpServer, Responder};
use actix_web::{HttpRequest, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Serialize, Deserialize)]
struct Book {
    title: String,
    title_en: String,
    author: String,
    ganre: String,
    sentences: [String; 3],
}
#[post("/echo")]
async fn echo(req_body: String) -> impl Responder {
    HttpResponse::Ok().body("test")
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
            .service(fs::Files::new("/static", "static"))
            .service(fs::Files::new("/static/svg", "static/css"))
            .route("/", web::get().to(index))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
