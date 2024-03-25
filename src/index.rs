use crate::book::Book;
use crate::db::get_entry;
use crate::handlers::*;
use crate::requests::NextReq;
use actix_files as fs;
use actix_session::Session;
use actix_web::{get, web, Error, HttpRequest, HttpResponse, Result};
use std::collections::HashMap;
use std::path::PathBuf;
use tera::Context;

#[get("/")]
pub async fn index(_req: HttpRequest) -> Result<fs::NamedFile, Error> {
    let path: PathBuf = "./static/index.html".parse()?;
    let file_result = fs::NamedFile::open(path).map_err(|e| {
        eprintln!("{}", e);
        actix_web::error::InternalError::new(
            "404 Not Found",
            actix_web::http::StatusCode::NOT_FOUND,
        )
    })?;
    Ok(file_result)
}

#[get("/api/index")]
pub async fn render_index(
    data: web::Data<Vec<Book>>,
    session: Session,
    params: web::Query<NextReq>,
) -> Result<HttpResponse, Error> {
    let mut context = Context::new();
    let db_size = data.len();
    let count = session.get::<usize>("counter")?.unwrap_or(0);

    if count + 1 >= db_size && params.next.unwrap_or(false)
        || session
            .get(&format!("book_{}_done", &db_size - 1))?
            .unwrap_or(false)
    {
        let mut books = HashMap::new();
        let all_points = session.get::<f32>("all_points")?.unwrap_or(0.00);
        let progress = (all_points / (db_size as f32 * 5.00)) * 100.00;
        context.insert("progress", &progress);
        context.insert("all_points", &all_points);
        for book_number in 0..db_size {
            let points = session
                .get::<f32>(format!("points_{}", book_number).as_str())?
                .unwrap_or(0.00);
            let book = get_entry(&data, book_number).await?;
            let key = format!("{} \"{}\"", book.author, book.title);
            books.insert(key, points);
        }
        context.insert("books", &books);
        let render = get_template("finish.html", context, &session);
        return parse_render(render);
    }
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
                &get_template("help1-avail.html", Context::new(), &session)?,
            );
            session.insert("help1_state", true)?;
            context.insert("help2_state", &"");
        }
        (false, true) => {
            context.insert("help1_state", &"");
            context.insert(
                "help2_state",
                &get_template("help2-avail.html", Context::new(), &session)?,
            );
            session.insert("help2_state", true)?;
        }
        (_, _) => {
            context.insert(
                "help1_state",
                &get_template("help1-avail.html", Context::new(), &session)?,
            );
            session.insert("help1_state", true)?;
            context.insert(
                "help2_state",
                &get_template("help2-avail.html", Context::new(), &session)?,
            );
            session.insert("help2_state", true)?;
        }
    }

    let all_points = session.get::<f32>("all_points")?.unwrap_or(0.00);
    context.insert("all_points", &all_points);
    let progress = (all_points / (data.len() as f32 * 5.00)) * 100.00;
    context.insert("progress", &progress);
    context.insert("counter", &session.get::<usize>("counter")?.unwrap_or(0));
    let render = get_template("index.html", context, &session);
    parse_render(render)
}
