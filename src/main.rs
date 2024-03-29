mod book;
mod config;
mod db;
mod endpoints;
mod handlers;
mod index;
mod messages;
mod requests;

use actix_files as fs;
use actix_session::{storage::CookieSessionStore, SessionMiddleware};
use actix_web::{cookie::Key, web, App, HttpServer};
use config::parse_config_or_exit;
use db::*;
use endpoints::*;
use index::*;

#[macro_use]
extern crate serde;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let config = parse_config_or_exit();
    let data = web::Data::new(
        load_data_from_file(&config.get_db_file()).expect("Failed to read database json file!"),
    );
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
            .service(change_lang)
            .service(index)
            .service(fs::Files::new("/static", "static"))
            .service(fs::Files::new("/static/svg", "static/css"))
    })
    .bind((config.get_bind_host(), config.get_bind_port()))?
    .run()
    .await
}
