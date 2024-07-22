use std::net::SocketAddr;
use chrono::{DateTime, Utc};
use sqlx::{Error, MySql, Pool, Row};
use sqlx::mysql::MySqlRow;
use tarpc::context::Context;
use crate::types::{Edit, Message, MessageData, Reaction, RealmChat, Redaction, Reply, Room, User};
use realm_shared::types::ErrorCode::*;
use realm_shared::types::ErrorCode;

#[derive(Clone)]
pub struct RealmChatServer {
	pub server_id: String,
	pub socket: SocketAddr, 
	pub db_pool: Pool<MySql>,
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
		let result = sqlx::query(
			"SELECT * FROM message INNER JOIN room ON message.room = room.id INNER JOIN user ON message.user = user.id WHERE message.id = ?"
		).bind(id).fetch_one(&self.db_pool).await;

		match result {
		    Ok(row) => {
				self.dbmessage_to_message(row)
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
		let result = sqlx::query("SELECT * FROM room").fetch_all(&self.db_pool).await;
		let mut rooms: Vec<Room> = Vec::new();

		match result {
		    Ok(rows) => {
				for row in rows {
					let room = self.dbroom_to_room(row);
					if let Some(err) = room.clone().err() {
						return Err(err)
					}
					rooms.push(room.unwrap());
				}
				Ok(rooms)
			},
			Err(_) => {
				Err(Error)
			},
		}
	}

	async fn get_room(self, _: Context, auth_token: String, roomid: String) -> Result<Room, ErrorCode> {
		//TODO: Auth for admin rooms!
		let result = sqlx::query("SELECT * FROM room WHERE room_id = ?").bind(roomid).fetch_one(&self.db_pool).await;
		
		match result {
		    Ok(row) => { self.dbroom_to_room(row) },
			Err(_) => Err(RoomNotFound),
		}
	}

	async fn get_user(self, _: Context, userid: String) -> Result<User, ErrorCode> {
		let result = sqlx::query("SELECT * FROM user WHERE user_id = ?").bind(userid).fetch_one(&self.db_pool).await;
		
		match result {
		    Ok(row) => { self.dbuser_to_user(row) },
			Err(_) => Err(UserNotFound),
		}
	}

	async fn get_users(self, _: Context, get_only_online: bool) -> Result<Vec<User>, ErrorCode> {
		let mut query = sqlx::query("SELECT * FROM user");
		if get_only_online {
			query = sqlx::query("SELECT * FROM user WHERE online = true");
		}
		
		let result = query.fetch_all(&self.db_pool).await;
		let mut users: Vec<User> = Vec::new();

		match result {
			Ok(rows) => {
				for row in rows {
					let user = self.dbuser_to_user(row);
					if let Some(err) = user.clone().err() {
						return Err(err)
					}
					users.push(user.unwrap())
				}
				Ok(users)
			},
			Err(_) => {
				Err(Error)
			},
		}	
	}
}

impl RealmChatServer {
	pub fn new(server_id: String, socket: SocketAddr, db_pool: Pool<MySql>) -> RealmChatServer {
		RealmChatServer {
			server_id,
			socket,
			db_pool,
			typing_users: Vec::new(),
		}
	}
	
	fn dbroom_to_room(&self, row: MySqlRow) -> Result<Room, ErrorCode> {
		let id: Result<u32, _> = row.try_get("id");
		let roomid: Result<String, _> = row.try_get("user_id");
		let name: Result<String, _> = row.try_get("name");
		let admin_only_send: Result<bool, _> = row.try_get("admin_only_send");
		let admin_only_view: Result<bool, _> = row.try_get("admin_only_view");

		if id.is_err() {
			return Err(MalformedDBResponse)
		}

		Ok(Room {
			id: id.unwrap(),
			roomid: roomid.unwrap(),
			name: name.unwrap(),
			admin_only_send: admin_only_send.unwrap(),
			admin_only_view: admin_only_view.unwrap(),
		})
	}
	
	fn dbuser_to_user(&self, row: MySqlRow) -> Result<User, ErrorCode> {
		let id: Result<u32, _> = row.try_get("id");
		let userid: Result<String, _> = row.try_get("user_id");
		let name: Result<String, _> = row.try_get("name");
		let online: Result<bool, _> = row.try_get("online");
		let admin: Result<bool, _> = row.try_get("admin");
		
		if id.is_err() {
			return Err(MalformedDBResponse)
		}
		
		Ok(User {
			id: id.unwrap(),
			userid: userid.unwrap(),
			name: name.unwrap(),
			online: online.unwrap(),
			admin: admin.unwrap(),
		})
	}
	
	fn dbmessage_to_message(&self, row: MySqlRow) -> Result<Message, ErrorCode> { //NOTE: Query results passed in should have a join
		let result: Result<&str, Error> = row.try_get("type");
		let type_enum: &str = match result {
			Ok(string) => { string }
			Err(_) => { "" }
		};

		if type_enum == "" {
			return Err(MalformedDBResponse)
		}

		let id: u32 = row.try_get("message.id").unwrap();
		let timestamp: DateTime<Utc> = row.try_get("timestamp").unwrap();
		
		let room = Room {
			id: row.try_get("room").unwrap(),
			roomid: row.try_get("room_id").unwrap(),
			name: row.try_get("room.name").unwrap(),
			admin_only_send: row.try_get("admin_only_send").unwrap(),
			admin_only_view: row.try_get("admin_only_view").unwrap(),
		};
		
		let user = User {
			id: row.try_get("user.id").unwrap(),
			userid: row.try_get("user_id").unwrap(),
			name: row.try_get("user.name").unwrap(),
			online: row.try_get("online").unwrap(),
			admin: row.try_get("admin").unwrap(),
		};

		match type_enum {
			"text" => {
				let text: String = row.try_get("msgText").unwrap();
				Ok(Message {
					id, timestamp, user, room,
					data: MessageData::Text(text),
				})
			}
			"attachment" => {
				todo!()
			}
			"reply" => {
				let text: &str = row.try_get("msgText").unwrap();
				let referencing_id: u32 = row.try_get("referencingID").unwrap();
				Ok(Message {
					id, timestamp, user, room,
					data: MessageData::Reply(Reply {
						referencing_id,
						text: text.to_string(),
					}),
				})
			}
			"edit" => {
				let text: &str = row.try_get("msgText").unwrap();
				let referencing_id: u32 = row.try_get("referencingID").unwrap();
				Ok(Message {
					id, timestamp, user, room,
					data: MessageData::Edit(Edit {
						referencing_id,
						text: text.to_string(),
					}),
				})
			}
			"reaction" => {
				let emoji: &str = row.try_get("emoji").unwrap();
				let referencing_id: u32 = row.try_get("referencingID").unwrap();
				Ok(Message {
					id, timestamp, user, room,
					data: MessageData::Reaction(Reaction {
						referencing_id,
						emoji: emoji.to_string(),
					}),
				})
			}
			"redaction" => {
				let referencing_id: u32 = row.try_get("referencingID").unwrap();
				Ok(Message {
					id, timestamp, user, room,
					data: MessageData::Redaction(Redaction {
						referencing_id,
					}),
				})
			}
			_ => { Err(MalformedDBResponse) }
		}
	}
}