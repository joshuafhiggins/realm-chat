use std::net::SocketAddr;

use futures::future;
use futures::future::Ready;
use sqlx::{MySql, Pool};
use tarpc::context::Context;
use tarpc::server::incoming::Incoming;
use crate::types::{ErrorCode, Message, MessageData, RealmChat, Room, User};

#[derive(Clone)]
pub struct RealmChatServer {
	pub socket: SocketAddr, 
	pub db_pool: Pool<MySql>,
}

impl RealmChatServer {
	pub fn new(socket: SocketAddr, db_pool: Pool<MySql>) -> RealmChatServer {
		RealmChatServer {
			socket,
			db_pool,
		}
	}
}

impl RealmChat for RealmChatServer {
	fn test(self, context: Context,  name: String) -> Ready<String> {
		future::ready(format!("Hello, {name}!"))
	}

	async fn send_message(self, context: Context, message: Message) -> Result<Message, ErrorCode> {
		//TODO: verify authentication somehow
		
		let result = match &message.data {
			MessageData::Text(text) => { 
				sqlx::query("INSERT INTO message (timestamp, user, room, type, msgText) VALUES (?, ?, ?, 'text', ?)")
					.bind(message.timestamp).bind(message.user.id).bind(message.room.id).bind(text)
					.execute(&self.db_pool).await 
			}
			MessageData::Attachment(attachment) => { todo!() }
			MessageData::Reply(reply) => {
				sqlx::query("INSERT INTO message (timestamp, user, room, type, msgText, referencingID) VALUES (?, ?, ?, 'reply', ?, ?)")
					.bind(message.timestamp).bind(message.user.id).bind(message.room.id).bind(reply.text.clone()).bind(reply.referencing_id)
					.execute(&self.db_pool).await
			}
			MessageData::Edit(edit) => {
				sqlx::query("INSERT INTO message (timestamp, user, room, type, msgText, referencingID) VALUES (?, ?, ?, 'edit', ?, ?)")
					.bind(message.timestamp).bind(message.user.id).bind(message.room.id).bind(edit.text.clone()).bind(edit.referencing_id)
					.execute(&self.db_pool).await
			}
			MessageData::Reaction(reaction) => {
				sqlx::query("INSERT INTO message (timestamp, user, room, type, emoji, referencingID) VALUES (?, ?, ?, 'reaction', ?, ?)")
					.bind(message.timestamp).bind(message.user.id).bind(message.room.id).bind(reaction.emoji.clone()).bind(reaction.referencing_id)
					.execute(&self.db_pool).await
			}
			MessageData::Redaction(redaction) => {
				sqlx::query("INSERT INTO message (timestamp, user, room, type, redaction, referencingID) VALUES (?, ?, ?, 'redaction', ?, ?)")
					.bind(message.timestamp).bind(message.user.id).bind(message.room.id).bind(true).bind(redaction.referencing_id)
					.execute(&self.db_pool).await
			}
		};
		
		match result {
		    Ok(ids) => {
				//TODO: Tell everyone

				Ok(message)
			},
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
		todo!()
	}

	async fn get_room(self, context: Context, roomid: String) -> Result<Room, ErrorCode> {
		todo!()
	}

	async fn get_user(self, context: Context, userid: String) -> Result<User, ErrorCode> {
		todo!()
	}

	async fn get_joined_users(self, context: Context) -> Result<Vec<User>, ErrorCode> {
		todo!()	
	}

	async fn get_online_users(self, context: Context) -> Result<Vec<User>, ErrorCode> {
		todo!()
	}
}