use std::env;
use std::future::Future;
use std::net::{IpAddr, Ipv6Addr};
use dotenvy::dotenv;
use futures::future::{self};
use futures::StreamExt;
use sqlx::mysql::MySqlPoolOptions;
use tarpc::{
    server::{Channel},
    tokio_serde::formats::Json,
};
use tarpc::server::incoming::Incoming;
use tarpc::server::BaseChannel;
use realm_server::server::RealmChatServer;
use realm_server::types::RealmChat;

async fn spawn(fut: impl Future<Output = ()> + Send + 'static) {
    tokio::spawn(fut);
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();
    
    let db_pool = MySqlPoolOptions::new()
        .max_connections(64)
        .connect(env::var("DATABASE_URL").expect("DATABASE_URL must be set").as_str()).await?;
    
    sqlx::query(
        "CREATE DATABASE IF NOT EXISTS realmchat; USE realmchat;"
    ).fetch_one(&db_pool).await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS room (
                id SERIAL,
                room_id VARCHAR(255) NOT NULL,
                name VARCHAR(255) NOT NULL,
                admin_only_send BOOL NOT NULL,
                admin_only_view BOOL NOT NULL
            );"
    ).execute(&db_pool).await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS user (
                id SERIAL,
                user_id VARCHAR(255) NOT NULL,
                name VARCHAR(255) NOT NULL,
                online BOOL NOT NULL,
                admin BOOL NOT NULL
            );"
    ).execute(&db_pool).await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS message (
                id SERIAL,
                timestamp DATETIME NOT NULL,
                user INT NOT NULL,
                room INT NOT NULL,
                type ENUM('text', 'attachment', 'reply', 'edit', 'reaction', 'redaction') NOT NULL,

                msgText TEXT,
                referencingID INT,
                emoji TEXT,
                redaction BOOL
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
            let server = RealmChatServer::new(env::var("SERVER_ID").expect("SERVER_ID must be set"), channel.transport().peer_addr().unwrap(), db_pool.clone());
            channel.execute(server.serve()).for_each(spawn)
        })
        // Max 10 channels.
        .buffer_unordered(10)
        .for_each(|_| async {})
        .await;

    Ok(())
}