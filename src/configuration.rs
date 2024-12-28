use crate::domain::subscriber_email::SubscriberEmail;
use secrecy::{ExposeSecret, SecretBox};
use std::time::Duration;

#[derive(Debug)]
pub enum Environment {
    Local,
    Production,
}

impl Environment {
    pub fn as_str(&self) -> &'static str {
        match self {
            Environment::Local => "local",
            Environment::Production => "production",
        }
    }
}

impl TryFrom<String> for Environment {
    type Error = String;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.to_lowercase().as_str() {
            "local" => Ok(Environment::Local),
            "production" => Ok(Environment::Production),
            _ => Err(format!("{} is not a supported environment", s)),
        }
    }
}

#[derive(serde::Deserialize)]
pub struct Settings {
    pub application: ApplicationSettings,
    pub database: DatabaseSettings,
    pub email_client: EmailClientSettings,
}

#[derive(serde::Deserialize)]
pub struct ApplicationSettings {
    pub port: u16,
    pub host: String,
}

#[derive(serde::Deserialize)]
pub struct EmailClientSettings {
    pub base_url: String,
    pub sender_email: String,
    pub authorization_token: SecretBox<String>,
    pub timeout_milliseconds: u64,
}

impl EmailClientSettings {
    pub fn sender(&self) -> Result<SubscriberEmail, String> {
        SubscriberEmail::parse(self.sender_email.clone())
    }

    pub fn timeout(&self) -> Duration {
        Duration::from_millis(self.timeout_milliseconds)
    }
}

#[derive(serde::Deserialize)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: SecretBox<String>,
    pub port: u16,
    pub host: String,
    pub database_name: String,
}

impl DatabaseSettings {
    /// 获取数据库连接字符串
    pub fn connection_string(&self) -> SecretBox<String> {
        SecretBox::new(Box::new(format!(
            "postgres://{}:{}@{}:{}/{}",
            self.username,
            self.password.expose_secret(),
            self.host,
            self.port,
            self.database_name
        )))
    }

    /// 获取数据库连接字符串，不包含数据库名称，连接到默认数据库，便于在测试时，切换数据库
    pub fn connection_string_without_db(&self) -> SecretBox<String> {
        SecretBox::new(Box::new(format!(
            "postgres://{}:{}@{}:{}",
            self.username,
            self.password.expose_secret(),
            self.host,
            self.port,
        )))
    }
}

///
/// 获取配置
///
/// 首先获取base.yaml, 作为基础变量。然后根据环境变量`APP_ENVIRONMENT`，获取对应的配置文件
/// 如果是本地环境，则读取`local.yaml`文件，如果是生产环境，则读取`production.yaml`文件
///
pub fn get_configuration() -> Result<Settings, config::ConfigError> {
    // 获取当前工作目录
    let base_path = std::env::current_dir().expect("Failed to determine the current directory");
    // 获取配置文件目录
    let configuration_directory = base_path.join("configuration");
    // 获取环境变量
    let environment: Environment = std::env::var("APP_ENVIRONMENT")
        .unwrap_or_else(|_| "local".into())
        .try_into()
        .expect("Failed to parse APP_ENVIRONMENT.");
    // 获取环境变量对应的配置文件
    let environment_filename = format!("{}.yaml", environment.as_str());
    // 创建配置对象
    let settings = config::Config::builder()
        // 添加基础配置文件
        .add_source(config::File::from(
            configuration_directory.join("base.yaml"),
        ))
        .add_source(config::File::from(
            configuration_directory.join(&environment_filename),
        ))
        .build()?;
    // 将配置文件反序列化为Settings结构体
    settings.try_deserialize::<Settings>()
}
