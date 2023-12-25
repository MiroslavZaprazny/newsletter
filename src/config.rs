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
    pub fn connection_string(&self) -> String {
        let require_ssl = if self.require_ssl {
            String::from("require")
        } else {
            String::from("prefer")
        };

        format!(
            "postgres://{}:{}@{}:{}/{}?sslmode={}",
            self.username, self.password, self.host, self.port, self.database_name, require_ssl
        )
    }

    pub fn without_db(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}",
            self.username, self.password, self.host, self.port
        )
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
