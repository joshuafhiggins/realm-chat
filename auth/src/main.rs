use std::env;
use std::future::Future;
use std::net::{IpAddr, Ipv6Addr};
use dotenvy::dotenv;
use futures::{future, StreamExt};
use sqlx::mysql::MySqlPoolOptions;
use tarpc::server::{BaseChannel, Channel};
use tarpc::server::incoming::Incoming;
use tarpc::tokio_serde::formats::Json;
use realm_auth::server::RealmAuthServer;
use realm_auth::types::{AuthEmail, RealmAuth};

async fn spawn(fut: impl Future<Output = ()> + Send + 'static) {
    tokio::spawn(fut);
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();

    let auth_email = AuthEmail {
        server_address: env::var("SERVER_MAIL_ADDRESS").expect("SERVER_MAIL_ADDRESS must be set"),
        server_port: env::var("SERVER_MAIL_PORT").expect("SERVER_MAIL_PORT must be set").parse::<u16>().expect("SERVER_MAIL_ADDRESS must be a number"),
        auth_name: env::var("SERVER_MAIL_NAME").expect("SERVER_MAIL_NAME must be set"),
        auth_from_address: env::var("SERVER_MAIL_FROM_ADDRESS").expect("SERVER_MAIL_FROM_ADDRESS must be set"),
        auth_username: env::var("SERVER_MAIL_USERNAME").expect("SERVER_MAIL_USERNAME must be set"),
        auth_password: env::var("SERVER_MAIL_PASSWORD").expect("SERVER_MAIL_PASSWORD must be set"),
    };

    let db_pool = MySqlPoolOptions::new()
        .max_connections(64)
        .connect(env::var("DATABASE_URL").expect("DATABASE_URL must be set").as_str()).await?;

    //TODO: In a docker container or figure out somewhere to do this command
    //sqlx::query("CREATE DATABASE IF NOT EXISTS realmauth").execute(&db_pool).await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS user (
                id SERIAL,
                username VARCHAR(255) NOT NULL,
                email VARCHAR(255) NOT NULL,
                new_email VARCHAR(255),
                avatar TEXT NOT NULL,
                login_code INT(6),
                tokens TEXT,
                google_oauth VARCHAR(255),
                apple_oauth VARCHAR(255),
                github_oauth VARCHAR(255),
                discord_oauth VARCHAR(255)
             );"
    ).execute(&db_pool).await?;

    let server_addr = (IpAddr::V6(Ipv6Addr::LOCALHOST), env::var("PORT").expect("PORT must be set").parse::<u16>().unwrap());

    // JSON transport is provided by the json_transport tarpc module. It makes it easy
    // to start up a serde-powered json serialization strategy over TCP.
    let mut listener = tarpc::serde_transport::tcp::listen(&server_addr, Json::default).await?;
    tracing::info!("Listening on port {}", listener.local_addr().port());
    listener.config_mut().max_frame_length(usize::MAX);
    listener
        // Ignore accept errors.
        .filter_map(|r| future::ready(r.ok()))
        .map(BaseChannel::with_defaults)
        // Limit channels to 1 per IP.
        .max_channels_per_key(1, |t| t.transport().peer_addr().unwrap().ip())
        // serve is generated by the service attribute. It takes as input any type implementing
        // the generated World trait.
        .map(|channel| {
            let server = RealmAuthServer::new(channel.transport().peer_addr().unwrap(), db_pool.clone(), auth_email.clone());
            channel.execute(server.serve()).for_each(spawn)
        })
        // Max 10 channels.
        .buffer_unordered(10)
        .for_each(|_| async {})
        .await;

    Ok(())
}