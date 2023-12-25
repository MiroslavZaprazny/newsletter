use sqlx::postgres::{PgConnectOptions, PgSslMode};

#[derive(serde::Deserialize)]
pub struct Settings {
    pub database: DatabaseSettings,
    pub application: ApplicationSettings,
}

#[derive(serde::Deserialize)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: String,
    pub port: String,
    pub host: String,
    pub database_name: String,
    pub require_ssl: bool,
}

#[derive(serde::Deserialize)]
pub struct ApplicationSettings {
    pub host: String,
    pub port: u16,
}

pub enum Enviroment {
    Local,
    Production,
}

impl DatabaseSettings {
    pub fn without_db(&self) -> PgConnectOptions{
        let require_ssl = if self.require_ssl {
            PgSslMode::Require
        } else {
            PgSslMode::Prefer
        };
        let port = self.port.parse::<u16>().expect("Failed to cast port to number");

        PgConnectOptions::new()
            .host(&self.host)
            .port(port)
            .username(&self.username)
            .password(&self.password)
            .ssl_mode(require_ssl)
    }

    pub fn with_db(&self) -> PgConnectOptions {
        self.without_db().database(&self.database_name)
    }
}

impl Enviroment {
    pub fn as_str(&self) -> &'static str {
        match self {
            Enviroment::Local => "local",
            Enviroment::Production => "production",
        }
    }
}

impl TryFrom<String> for Enviroment {
    type Error = String;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            "local" => Ok(Self::Local),
            "production" => Ok(Self::Production),
            other => Err(format!("{} is not a valid enviroment", other)),
        }
    }
}

pub fn get_config() -> Result<Settings, config::ConfigError> {
    let base_dir = std::env::current_dir().expect("Failed to retrieve current directory");
    let config_dir = base_dir.join("configuration");
    let enviroment: Enviroment = std::env::var("APP_ENVIRONMENT")
        .unwrap_or_else(|_| {
            "local"
                .try_into()
                .expect("Failed to parse local enviroment")
        })
        .try_into()
        .expect("Failed to parse APP_ENVIROMENT");

    let env_file = format!("{}.yaml", enviroment.as_str());
    let settings = config::Config::builder()
        .add_source(config::File::from(config_dir.join("base.yaml")))
        .add_source(config::File::from(config_dir.join(env_file)))
        .add_source(
            config::Environment::with_prefix("APP")
                .prefix_separator("_")
                .separator("__"),
        )
        .build()?;

    settings.try_deserialize::<Settings>()
}
