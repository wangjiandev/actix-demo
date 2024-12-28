use actix_demo::{
    configuration::{get_configuration, DatabaseSettings},
    email_client::EmailClient,
    startup::run,
    telemetry::{get_subscriber, init_subscriber},
};
use once_cell::sync::Lazy;
use secrecy::ExposeSecret;
use sqlx::{Connection, PgConnection, PgPool};
use std::net::TcpListener;
use uuid::Uuid;

/// 初始化日志
/// 确保使用once_cell::sync::Lazy，确保只初始化一次
static TRACING: Lazy<()> = Lazy::new(|| {
    // 设置日志级别
    let default_filter_level = "info".to_string();
    // 设置日志订阅者名称
    let subscriber_name = "test".to_string();
    // 如果环境变量TEST_LOG为true，则将日志输出到控制台
    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::stdout);
        init_subscriber(subscriber);
    } else {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::sink);
        init_subscriber(subscriber);
    }
});

pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
}

pub async fn spawn_app() -> TestApp {
    // 初始化日志
    Lazy::force(&TRACING);

    // 创建TCP监听器
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    // 获取监听器端口
    let port = listener.local_addr().unwrap().port();
    // 创建服务器地址
    let address = format!("http://127.0.0.1:{}", port);

    // 获取配置文件
    let mut configuration = get_configuration().expect("Failed to read configuration.");
    // 创建测试数据库
    configuration.database.database_name = Uuid::new_v4().to_string();
    // 创建数据库连接池
    let connection_pool = configure_database(&configuration.database).await;
    // 获取邮件发送客户端
    let sender_email = configuration
        .email_client
        .sender()
        .expect("Invalid sender email address.");
    // 获取邮件发送超时时间
    let timeout = configuration.email_client.timeout();
    // 创建邮件发送客户端
    let email_client = EmailClient::new(
        configuration.email_client.base_url,
        sender_email,
        configuration.email_client.authorization_token,
        timeout,
    );

    // 创建服务器
    let server = run(listener, connection_pool.clone(), email_client).expect("Failed to spawn app");
    // 启动服务器
    let _ = tokio::spawn(server);
    // 返回测试应用
    TestApp {
        address,
        db_pool: connection_pool,
    }
}

async fn configure_database(config: &DatabaseSettings) -> PgPool {
    // 创建数据库
    let mut connection =
        PgConnection::connect(&config.connection_string_without_db().expose_secret())
            .await
            .expect("Failed to connect to the postgres database");
    sqlx::query(format!(r#"CREATE DATABASE "{}";"#, config.database_name).as_str())
        .execute(&mut connection)
        .await
        .expect("Failed to create database.");

    // 迁移数据
    let connection_pool = PgPool::connect(&config.connection_string().expose_secret())
        .await
        .expect("Failed to connect to the postgres database");
    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to migrate the database");

    connection_pool
}
