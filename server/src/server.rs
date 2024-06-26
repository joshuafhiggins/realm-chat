use std::net::SocketAddr;
use std::sync::Arc;
use futures::future;
use futures::future::Ready;
use surrealdb::engine::remote::ws::Client;
use surrealdb::{Error, Response, Surreal};
use tarpc::context::Context;
use crate::types::{Message, RealmChat, Room, ErrorCode, User, Record};

#[derive(Clone)]
pub struct RealmChatServer {
	pub socket: SocketAddr, 
	pub db: Arc<Surreal<Client>>,
}

impl RealmChatServer {
	pub fn new(socket: SocketAddr, db: Arc<Surreal<Client>>) -> RealmChatServer {
		RealmChatServer {
			socket,
			db,
		}
	}
}

impl RealmChat for RealmChatServer {
	fn test(self, context: Context,  name: String) -> Ready<String> {
		future::ready(format!("Hello, {name}!"))
	}

	async fn send_message(self, context: Context, message: Message) -> Result<Message, ErrorCode> {
		let created: surrealdb::Result<Vec<Record>> = self.db
			.create(message.room.roomid.clone())
			.content(message.clone())
			.await;
		
		//TODO: Tell everyone
		
		match created {
		    Ok(ids) => Ok(message),
			Err(_) => Err(ErrorCode::Error),
		}
	}

	async fn start_typing(self, context: Context) -> ErrorCode {
		todo!()
	}

	async fn stop_typing(self, context: Context) -> ErrorCode {
		todo!()
	}

	async fn keep_typing(self, context: Context) -> ErrorCode {
		todo!()
	}

	async fn get_message_from_guid(self, context: Context, guid: String) -> Result<Message, ErrorCode> {
		todo!()
	}

	async fn get_messages_since(self, context: Context, time: u64) -> Result<Vec<Message>, ErrorCode> {
		todo!()
	}

	async fn get_rooms(self, context: Context) -> Result<Vec<Room>, ErrorCode> {
		let result: surrealdb::Result<Vec<Room>> = self.db.select("room").await;
		
		match result {
			Ok(rooms) => Ok(rooms),
			Err(_) => Err(ErrorCode::Error),
		}
	}

	async fn get_room(self, context: Context, roomid: String) -> Result<Room, ErrorCode> {
		todo!()
	}

	async fn get_user(self, context: Context, userid: String) -> Result<User, ErrorCode> {
		todo!()
	}

	async fn get_joined_users(self, context: Context) -> Result<Vec<User>, ErrorCode> {
		let result: surrealdb::Result<Vec<User>> = self.db.select("user").await;

		match result {
			Ok(users) => Ok(users),
			Err(_) => Err(ErrorCode::Error),
		}	
	}

	async fn get_online_users(self, context: Context) -> Result<Vec<User>, ErrorCode> {
		let result: surrealdb::Result<Response> = self.db.query("SELECT * FROM user WHERE online = true").await; //TODO: We're switching to MySQL

		match result {
			Ok(mut response) => {
				let users: Result<Vec<User>, Error> = response.take(0);
				match users {
					Ok(vec) => Ok(vec),
					Err(_) => Err(ErrorCode::Error),
				}
			},
			Err(_) => Err(ErrorCode::Error),
		}
	}
}