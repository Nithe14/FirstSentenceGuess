use crate::book::Book;
use crate::db::get_entry;
use crate::handlers::*;
use crate::messages::parse_messages;
use crate::requests::*;
use actix_session::Session;
use actix_web::{get, post, web, Error, HttpResponse, Result};
use tera::Context;

#[get("/api/counter")]
pub async fn counter(data: web::Data<Vec<Book>>, session: Session) -> Result<HttpResponse, Error> {
    let db_size = data.len();
    if let Some(count) = session.get::<usize>("counter")? {
        if count < db_size - 1 {
            session.insert("counter", count + 1)?;
        }
    } else {
        session.insert("counter", 0)?;
    }

    let book_counter = session.get::<usize>("counter")?.unwrap_or_default();
    Ok(HttpResponse::Ok()
        .insert_header(("HX-Trigger", "newBook"))
        .body(format!("{}/10", book_counter + 1)))
}

#[post("/api/give-up")]
pub async fn give_up(data: web::Data<Vec<Book>>, session: Session) -> Result<HttpResponse, Error> {
    let count = session.get::<usize>("counter")?.unwrap_or(0);
    session.insert(format!("points_{}", count.to_string()), 0)?;
    session.insert(format!("book_{}_done", count), true)?;
    let _ = session.insert("sentences_state", 1);
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
    render = get_template("give-up.html", context, &session);
    parse_render(render)
}

#[post("/api/check-book")]
pub async fn check_book(
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
        let _ = session.insert("sentences_state", 1);
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
        render = get_template("correct.html", context, &session);
        return parse_render(render);
    }

    if current_points > 1.00 {
        session.insert("current_points", (current_points - 1.00) as u8)?;
    }
    render = get_template("wrong.html", context, &session);
    match render {
        Ok(r) => Ok(HttpResponse::Ok()
            .insert_header(("HX-Retarget", "#frm")) //cannot use parse_render() because of custom headers
            .insert_header(("HX-Reswap", "outerHTML"))
            .body(r)),
        Err(e) => {
            eprintln!("{}", e);
            Ok(HttpResponse::InternalServerError().body("500 Internal Server Error"))
        }
    }
}

#[get("/api/sentences")]
pub async fn sentences(
    data: web::Data<Vec<Book>>,
    session: Session,
    params: web::Query<NextReq>,
) -> Result<HttpResponse, Error> {
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
            return Ok(HttpResponse::InternalServerError().body("500 Internal Server Error"));
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
            let render = get_template("sentence2.html", context, &session);
            parse_render(render)
        }
        3 => {
            session.insert("current_points", current_points)?;
            context.insert("sentence1", &book.sentences[0]);
            context.insert("sentence2", &book.sentences[1]);
            context.insert("sentence3", &book.sentences[2]);
            let render = get_template("sentence3.html", context, &session);
            parse_render(render)
        }
        _ => {
            context.insert("sentence1", &book.sentences[0]);
            context.insert("sentence2", &sentence2_placeholder);
            context.insert("sentence3", &sentence3_placeholder);
            let render = get_template("sentence1.html", context, &session);
            parse_render(render)
        }
    }
}

#[get("/api/get-help")]
pub async fn get_help(
    data: web::Data<Vec<Book>>,
    session: Session,
    params: web::Query<HelpReq>,
) -> Result<HttpResponse, Error> {
    let count = session.get::<usize>("counter")?.unwrap_or(0);
    let current_points = session.get::<u8>("current_points")?.unwrap_or(0);
    let book: Book = match get_entry(&data, count).await {
        Ok(value) => value,
        Err(e) => {
            eprintln!("{}", e);
            return Ok(HttpResponse::InternalServerError().body("500 Internal Server Error"));
        }
    };
    let mut context = Context::new();
    if current_points > 1 {
        session.insert("current_points", current_points - 1)?;
    }
    match params.number {
        1 => {
            session.insert("help1_state", false)?;
            context.insert("help1", &book.ganre);
            let render = get_template("help1.html", context, &session);
            parse_render(render)
        }
        _ => {
            session.insert("help2_state", false)?;
            context.insert("help2", &book.author);
            let render = get_template("help2.html", context, &session);
            parse_render(render)
        }
    }
}

#[get("/api/change-lang")]
pub async fn change_lang(params: web::Query<Lang>) -> HttpResponse {
    if parse_messages(&params.lang).is_ok() {
        HttpResponse::Ok()
            .insert_header(("HX-Refresh", "true"))
            .body("")
    } else {
        HttpResponse::BadRequest().body("400 Bad Request. Language not supported.")
    }
}
