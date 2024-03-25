use serde::Deserialize;

#[derive(Deserialize)]
pub struct Config {
    bind_host: Option<String>,
    bind_port: Option<u16>,
    db_file: String,
    db_safe: Option<bool>,
}

impl Config {
    pub fn get_bind_host(&self) -> String {
        self.bind_host
            .to_owned()
            .unwrap_or(String::from("127.0.0.1"))
    }
    pub fn get_bind_port(&self) -> u16 {
        self.bind_port.unwrap_or(8080)
    }
    pub fn get_db_file(&self) -> String {
        self.db_file.to_owned()
    }
    pub fn is_db_safe(&self) -> bool {
        self.db_safe.unwrap_or(false)
    }
}

pub fn parse_config_or_exit() -> Config {
    let config_path = std::env::current_dir()
        .unwrap_or_else(|e| {
            eprintln!("{}", e);
            std::process::exit(1);
        })
        .join("config.toml");
    let config_string = std::fs::read_to_string(&config_path).map_or_else(
        |e| {
            eprintln!("Failed to read config file: {}", e);
            std::process::exit(1);
        },
        |s| s,
    );
    match toml::from_str(&config_string) {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Failed to parse config file: {}", e);
            std::process::exit(1);
        }
    }
}
