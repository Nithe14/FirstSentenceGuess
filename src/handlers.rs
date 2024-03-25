use actix_session::Session;
use actix_web::HttpResponse;
use rand::{distributions::Alphanumeric, Rng};
use std::fmt::Display;
use tera::{Context, Tera};

use crate::config::parse_config_or_exit;
use crate::messages::parse_messages;

pub fn get_template(
    template: &str,
    mut context: Context,
    session: &Session,
) -> Result<String, Box<dyn std::error::Error>> {
    let tera = Tera::new("templates/*.html").map_err(|e| {
        eprintln!("Parsing error: {}", e);
        Box::new(e) as Box<dyn std::error::Error>
    })?;
    let is_safe = parse_config_or_exit().is_db_safe();
    let lang = session.get("lang")?.unwrap_or(String::from("en"));
    let messages = parse_messages(&lang)?;
    context.insert("is_safe", &is_safe);
    context.insert("messages", &messages);
    Ok(tera.render(template, &context)?)
}

pub fn parse_render<E>(render: Result<String, impl Display>) -> Result<HttpResponse, E>
where
    E: Display,
{
    render.map(|r| HttpResponse::Ok().body(r)).or_else(|e| {
        eprintln!("{}", e);
        Ok(HttpResponse::InternalServerError().body("500 Internal Server Error"))
    })
}

pub fn generate_placeholder(input: &str) -> String {
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
