use std::fmt::format;

#[derive(serde::Deserialize)]
pub struct Settings {
    pub database: DatabaseSettings,
    pub application_settings: ApplicationSettings
}

#[derive(serde::Deserialize)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: String,
    pub port: u16,
    pub host: String,
    pub database_name: String,
}

#[derive(serde::Deserialize)]
pub struct ApplicationSettings {
    pub host: String,
    pub port: u16
}

pub enum Enviroment {
    Local,
    Production
}

impl DatabaseSettings {
    pub fn connection_string(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.username, self.password, self.host, self.port, self.database_name
        )
    }

    pub fn connection_string_without_db(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}",
            self.username, self.password, self.host, self.port
        )
    }
}

impl Enviroment {
    pub fn as_str(&self) -> &'static str {
        return match self {
            Enviroment::Local => "local",
            Enviroment::Production => "production"
        }
    }
}

impl TryFrom<String> for Enviroment {
    type Error = String;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            "local" => Ok(Self::Local),
            "production" => Ok(Self::Production),
            other => Err(format!("{} is not a valid enviroment", other))
        }
    }
}

pub fn get_config() -> Result<Settings, config::ConfigError> {
    let settings = config::Config::builder();
    let base_dir = std::env::current_dir().expect("Failed to retrieve current directory");
    let config_dir = base_dir.join("configuration");

    let enviroment: Enviroment = std::env::var("APP_ENVIROMENT").unwrap_or_else(|_| "local".try_into().expect("Failed to parse local enviroment")).try_into().expect("Failed to parse APP_ENVIROMENT");
    settings.add_source(config::File::new(config_dir.join(enviroment.as_str()).to_str().unwrap(), config::FileFormat::Yaml));
}

