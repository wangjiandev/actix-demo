use actix_demo::{
    configuration::get_configuration,
    email_client::EmailClient,
    startup::run,
    telemetry::{get_subscriber, init_subscriber},
};
use secrecy::ExposeSecret;
use sqlx::PgPool;
use std::net::TcpListener;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // 配置logger
    let subscriber = get_subscriber("actix-demo".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);
    // 获取配置
    let configuration = get_configuration().expect("Failed to read configuration.");
    // 根据配置文件获取发送者邮箱地址
    let sender = configuration
        .email_client
        .sender()
        .expect("Invalid sender email address.");
    // 根据配置文件获取邮件发送超时时间
    let timeout = configuration.email_client.timeout();
    // 创建邮件发送客户端
    let email_client = EmailClient::new(
        configuration.email_client.base_url,
        sender,
        configuration.email_client.authorization_token,
        timeout,
    );
    // 创建服务器地址
    let address = format!(
        "{}:{}",
        configuration.application.host, configuration.application.port
    );
    // 创建TCP监听器
    let listener = TcpListener::bind(address)?;
    // 创建数据库连接池
    let connection_pool =
        PgPool::connect_lazy(&configuration.database.connection_string().expose_secret())
            .expect("Failed to connect to the database");
    // 运行服务器
    run(listener, connection_pool, email_client)?.await?;
    Ok(())
}
