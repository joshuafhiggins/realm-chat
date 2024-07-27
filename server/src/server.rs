use std::net::SocketAddr;

use chrono::{DateTime, Utc};
use sqlx::{FromRow, Pool, query_as, Sqlite};
use sqlx::query;
use tarpc::context::Context;

use realm_shared::types::ErrorCode::*;
use realm_shared::types::ErrorCode;

use crate::types::{Message, MessageData, RealmChat, Room, User};

#[derive(Clone)]
pub struct RealmChatServer {
	pub server_id: String,
	pub socket: SocketAddr, 
	pub db_pool: Pool<Sqlite>,
	pub typing_users: Vec<(u32, u32)> //NOTE: userid, roomid
} //TODO: Cache for auth

impl RealmChat for RealmChatServer {
	async fn test(self, _: Context, name: String) -> String {
		format!("Hello, {name}!")
	}

	async fn send_message(self, _: Context, auth_token: String, message: Message) -> Result<Message, ErrorCode> {
		//TODO: verify authentication somehow for edits and redactions
		
		let result = match &message.data {
			MessageData::Text(text) => { 
				query!("INSERT INTO message (timestamp, user, room, msg_type, msg_text) VALUES (?, ?, ?, 'text', ?)", 
					message.timestamp, message.user.id, message.room.id, text)
					.execute(&self.db_pool).await 
			}
			MessageData::Attachment(attachment) => { todo!() }
			MessageData::Reply(reply) => {
				query!("INSERT INTO message (timestamp, user, room, msg_type, msg_text, referencing_id) VALUES (?, ?, ?, 'reply', ?, ?)",
					message.timestamp, message.user.id, message.room.id, reply.text, reply.referencing_id)
					.execute(&self.db_pool).await
			}
			MessageData::Edit(edit) => {
				query!("INSERT INTO message (timestamp, user, room, msg_type, msg_text, referencing_id) VALUES (?, ?, ?, 'edit', ?, ?)",
					message.timestamp, message.user.id, message.room.id, edit.text, edit.referencing_id)
					.execute(&self.db_pool).await
			}
			MessageData::Reaction(reaction) => {
				query!("INSERT INTO message (timestamp, user, room, msg_type, emoji, referencing_id) VALUES (?, ?, ?, 'reaction', ?, ?)",
					message.timestamp, message.user.id, message.room.id, reaction.emoji, reaction.referencing_id)
					.execute(&self.db_pool).await
			}
			MessageData::Redaction(redaction) => {
				query!("INSERT INTO message (timestamp, user, room, msg_type, referencing_id) VALUES (?, ?, ?, 'redaction', ?)",
					message.timestamp, message.user.id, message.room.id, redaction.referencing_id)
					.execute(&self.db_pool).await
			}
		};
		
		match result {
		    Ok(ids) => {
				//TODO: Tell everyone

				Ok(message)
			},
			Err(_) => Err(Error),
		}
	}

	async fn start_typing(self, _: Context, auth_token: String) -> ErrorCode { //TODO: auth for all of these
		todo!()
	}

	async fn stop_typing(self, _: Context, auth_token: String) -> ErrorCode {
		todo!()
	}

	async fn keep_typing(self, _: Context, auth_token: String) -> ErrorCode {
		todo!()
	}

	async fn get_message_from_id(self, _: Context, auth_token: String, id: u32) -> Result<Message, ErrorCode> {
		//TODO: Auth for admin room
		let result = sqlx::query("SELECT message.*,
        room.id AS 'room_id', room.roomid AS 'room_roomid', room.name AS 'room_name', room.admin_only_send AS 'room_admin_only_send', room.admin_only_view AS 'room_admin_only_view',
        user.id AS 'user_id', user.userid AS 'user_userid', user.name AS 'user_name', user.online AS 'user_online', user.admin AS 'user_admin'
	    FROM message INNER JOIN room ON message.room = room.id INNER JOIN user ON message.user = user.id WHERE message.id = ?")
			.bind(id)
			.fetch_one(&self.db_pool).await;

		match result {
		    Ok(row) => {
				Ok(Message::from_row(&row).unwrap())
			},
			Err(_) => {
				Err(MessageNotFound)
			},
		}
	}

	async fn get_messages_since(self, _: Context, auth_token: String, time: DateTime<Utc>) -> Result<Vec<Message>, ErrorCode> {
		//TODO: Auth for admin rooms
		todo!()
	}

	async fn get_rooms(self, _: Context, auth_token: String) -> Result<Vec<Room>, ErrorCode> {
		//TODO: Auth for admin rooms!
		let result = query_as!(Room, "SELECT * FROM room").fetch_all(&self.db_pool).await;

		match result {
		    Ok(rooms) => Ok(rooms),
			Err(_) => Err(Error),
		}
	}

	async fn get_room(self, _: Context, auth_token: String, roomid: String) -> Result<Room, ErrorCode> {
		//TODO: Auth for admin rooms!
		let result = query_as!(Room, "SELECT * FROM room WHERE roomid = ?", roomid).fetch_one(&self.db_pool).await;
		
		match result {
		    Ok(room) => { Ok(room) },
			Err(_) => Err(RoomNotFound),
		}
	}

	async fn get_user(self, _: Context, userid: String) -> Result<User, ErrorCode> {
		let result = query_as!(User, "SELECT * FROM user WHERE userid = ?", userid).fetch_one(&self.db_pool).await;
		
		match result {
		    Ok(user) => { Ok(user) },
			Err(_) => Err(UserNotFound),
		}
	}

	async fn get_users(self, _: Context) -> Result<Vec<User>, ErrorCode> {
		let result = query_as!(User, "SELECT * FROM user").fetch_all(&self.db_pool).await;

		match result {
			Ok(users) => Ok(users),
			Err(_) => Err(Error),
		}
	}

	async fn get_online_users(self, _: Context) -> Result<Vec<User>, ErrorCode> {
		let result = query_as!(User, "SELECT * FROM user WHERE online = true").fetch_all(&self.db_pool).await;

		match result {
			Ok(users) => Ok(users),
			Err(_) => Err(Error),
		}
	}
}

impl RealmChatServer {
	pub fn new(server_id: String, socket: SocketAddr, db_pool: Pool<Sqlite>) -> RealmChatServer {
		RealmChatServer {
			server_id,
			socket,
			db_pool,
			typing_users: Vec::new(),
		}
	}
}