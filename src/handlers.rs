use actix_web::HttpResponse;
use rand::{distributions::Alphanumeric, Rng};
use std::fmt::Display;
use tera::{Context, Tera};

pub fn get_template(
    template: &str,
    context: Context,
) -> Result<String, Box<dyn std::error::Error>> {
    let tera = Tera::new("templates/pl/*").map_err(|e| {
        eprintln!("Parsing error: {}", e);
        Box::new(e) as Box<dyn std::error::Error>
    })?;
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
