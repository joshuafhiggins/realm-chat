use std::env;
use std::future::Future;
use std::net::{IpAddr, Ipv6Addr, SocketAddr};
use std::sync::{Arc};
use dotenvy::dotenv;
use durian::{PacketManager, ServerConfig};
use futures::future::{self};
use futures::StreamExt;
use sqlx::migrate::MigrateDatabase;
use sqlx::{migrate, Sqlite, SqlitePool};
use tarpc::{
	server::{Channel},
	tokio_serde::formats::Json,
};
use tarpc::server::incoming::Incoming;
use tarpc::server::BaseChannel;
use tokio::sync::Mutex;
use tracing::{info, subscriber, warn};
use realm_server::events::*;
use realm_server::server::RealmChatServer;
use realm_server::types::RealmChat;

async fn spawn(fut: impl Future<Output = ()> + Send + 'static) {
	tokio::spawn(fut);
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
	dotenv().ok();

	let subscriber = tracing_subscriber::fmt()
		.compact()
		.with_file(true)
		.with_line_number(true)
		.with_thread_ids(true)
		.with_target(false)
		.finish();

	subscriber::set_global_default(subscriber)?;

	let database_url: &str = &env::var("DATABASE_URL").expect("DATABASE_URL must be set");

	if !Sqlite::database_exists(database_url).await.unwrap_or(false) {
		info!("Creating database {}", database_url);
		match Sqlite::create_database(database_url).await {
			Ok(_) => info!("Create db success"),
			Err(error) => panic!("error: {}", error),
		}
	} else {
		warn!("Database already exists");
	} // TODO: Do in Docker with Sqlx-cli

	let db_pool = SqlitePool::connect(database_url).await?;

	info!("Running migrations...");
	migrate!().run(&db_pool).await?; // TODO: Do in Docker with Sqlx-cli
	info!("Migrations complete!");

	let port = env::var("PORT").expect("PORT must be set").parse::<u16>()?;
	let server_addr = (IpAddr::V6(Ipv6Addr::LOCALHOST), port);

	let mut inner_manager = PacketManager::new();

	inner_manager.register_receive_packet::<Greet>(GreetPacketBuilder)?;
	inner_manager.register_receive_packet::<UserJoinedEvent>(UserJoinedEventPacketBuilder)?;
	inner_manager.register_receive_packet::<UserLeftEvent>(UserLeftEventPacketBuilder)?;
	inner_manager.register_receive_packet::<NewMessageEvent>(NewMessageEventPacketBuilder)?;
	// inner_manager.register_receive_packet::<StartTypingEvent>(StartTypingEventPacketBuilder)?;
	// inner_manager.register_receive_packet::<StopTypingEvent>(StopTypingEventPacketBuilder)?;
	inner_manager.register_receive_packet::<NewRoomEvent>(NewRoomEventPacketBuilder)?;
	inner_manager.register_receive_packet::<DeleteRoomEvent>(DeleteRoomEventPacketBuilder)?;
	inner_manager.register_receive_packet::<KickedUserEvent>(KickedUserEventPacketBuilder)?;
	inner_manager.register_receive_packet::<BannedUserEvent>(BannedUserEventPacketBuilder)?;
	
	inner_manager.register_send_packet::<Greet>()?;
	inner_manager.register_send_packet::<UserJoinedEvent>()?;
	inner_manager.register_send_packet::<UserLeftEvent>()?;
	inner_manager.register_send_packet::<NewMessageEvent>()?;
	// inner_manager.register_send_packet::<StartTypingEvent>()?;
	// inner_manager.register_send_packet::<StopTypingEvent>()?;
	inner_manager.register_send_packet::<NewRoomEvent>()?;
	inner_manager.register_send_packet::<DeleteRoomEvent>()?;
	inner_manager.register_send_packet::<KickedUserEvent>()?;
	inner_manager.register_send_packet::<BannedUserEvent>()?;
	
	inner_manager.init_server(
		ServerConfig::new(
			SocketAddr::from((IpAddr::V6(Ipv6Addr::LOCALHOST), port-1)).to_string(),
			0, None, 8, 8))?;
	
	let manager = Arc::new(Mutex::new(inner_manager));
	info!("Listening on port {}", port-1);

	// JSON transport is provided by the json_transport tarpc module. It makes it easy
	// to start up a serde-powered json serialization strategy over TCP.
	let mut listener = tarpc::serde_transport::tcp::listen(&server_addr, Json::default).await?;
	info!("Listening on port {}", listener.local_addr().port());
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
			let server = RealmChatServer::new(env::var("SERVER_ID").expect("SERVER_ID must be set"), channel.transport().peer_addr().unwrap(), db_pool.clone(), manager.clone());
			channel.execute(server.serve()).for_each(spawn)
		})
		// Max 10 channels.
		.buffer_unordered(10)
		.for_each(|_| async {})
		.await;

	Ok(())
}