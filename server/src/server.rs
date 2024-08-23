use std::env;
use std::net::SocketAddr;
use std::time::Duration;
use chrono::{DateTime, Utc};
use moka::future::Cache;
use sqlx::{FromRow, Pool, query_as, Sqlite};
use sqlx::query;
use sqlx::sqlite::SqliteRow;
use tarpc::context::Context;
use tracing::error;
use realm_auth::types::RealmAuthClient;
use realm_shared::types::ErrorCode::*;
use realm_shared::types::ErrorCode;

use crate::types::{Attachment, Edit, FromRows, Message, MessageData, Reaction, RealmChat, Redaction, Reply, ReplyChain, Room, User};

#[derive(Clone)]
pub struct RealmChatServer {
	pub server_id: String,
	pub domain: String,
	pub port: u16,
	pub socket: SocketAddr, 
	pub db_pool: Pool<Sqlite>,
	pub typing_users: Vec<(String, String)>, //NOTE: user.userid, room.roomid
	pub auth_client: RealmAuthClient,
	pub cache: Cache<String, String>,
}

const FETCH_MESSAGE: &str = "SELECT message.*,
        room.id AS 'room_id', room.roomid AS 'room_roomid', room.name AS 'room_name', room.admin_only_send AS 'room_admin_only_send', room.admin_only_view AS 'room_admin_only_view',
        user.id AS 'user_id', user.userid AS 'user_userid', user.name AS 'user_name', user.online AS 'user_online', user.admin AS 'user_admin'
	    FROM message INNER JOIN room ON message.room = room.id INNER JOIN user ON message.user = user.id WHERE room.admin_only_view = ? OR false";

impl RealmChatServer {
	pub fn new(server_id: String, socket: SocketAddr, db_pool: Pool<Sqlite>, auth_client: RealmAuthClient) -> RealmChatServer {
		RealmChatServer {
			server_id,
			port: env::var("PORT").unwrap().parse::<u16>().unwrap(),
			domain: env::var("DOMAIN").expect("DOMAIN must be set"),
			socket,
			db_pool,
			typing_users: Vec::new(),
			auth_client,
			cache: Cache::builder()
				.max_capacity(10_000)
				.time_to_idle(Duration::from_secs(5*60))
				.time_to_live(Duration::from_secs(60*60))
				.build()
		}
	}
	
	pub async fn is_stoken_valid(&self, userid: &str, stoken: &str) -> bool {
		match self.cache.get(stoken).await {
		    None => {
				let result = self.auth_client.server_token_validation(
					tarpc::context::current(), stoken.to_string(), userid.to_string(), self.server_id.clone(), self.domain.clone(), self.port)
					.await;
				
				match result {
					Ok(valid) => {
						if valid {
							self.cache.insert(stoken.to_string(), userid.to_string()).await;
							return true
						}
						false
					}
					Err(_) => {
						error!("Error validating server token for user, {}, with stoken {}", userid, stoken);
						false
					}
				}
			}
			Some(cached_username) => cached_username.eq(userid),
		}
	}

	pub async fn is_user_admin(&self, stoken: &str) -> bool {
		if let Some(userid) = self.cache.get(stoken).await {
			let result = query!("SELECT admin FROM user WHERE userid = ?", userid).fetch_one(&self.db_pool).await;
			return match result {
				Ok(record) => {
					if record.admin {
						return true
					}
					false
				}
				Err(_) => false
			}
		}
		false
	}
}

impl RealmChat for RealmChatServer {
	async fn test(self, _: Context, name: String) -> String {
		format!("Hello, {name}!")
	}

	async fn send_message(self, ctx: Context, stoken: String, mut message: Message) -> Result<Message, ErrorCode> {
		if !self.is_stoken_valid(&message.user.userid, &stoken).await { // Check sender userid
			return Err(Unauthorized)
		}
		
		// Assert all the data in message is correct
		message.user = self.clone().get_user(ctx, message.user.userid).await.unwrap();
		
		match &message.data { // Check that the sender is the owner of the referencing msg
			MessageData::Edit(e) => {
				let ref_msg = self.clone().get_message_from_id(ctx, stoken.clone(), e.referencing_id).await?;
				if !ref_msg.user.userid.eq(&message.user.userid) {
					return Err(Unauthorized)
				}
			}
			MessageData::Redaction(r)=> {
				let ref_msg = self.clone().get_message_from_id(ctx, stoken.clone(), r.referencing_id).await?;
				if !ref_msg.user.userid.eq(&message.user.userid) {
					return Err(Unauthorized)
				}
			}
			_ => {}
		}

		let is_admin = self.is_user_admin(&stoken).await;
		let admin_only_send = query!(
			"SELECT admin_only_send FROM room WHERE roomid = ?",
			message.room.roomid).fetch_one(&self.db_pool).await;
		if let Ok(record) = admin_only_send {
			if record.admin_only_send && !is_admin {
				return Err(Unauthorized)
			}
		} else {
			return Err(RoomNotFound)
		}

		message.room = self.clone().get_room(ctx, stoken.clone(), message.room.roomid).await.unwrap();

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

	async fn start_typing(self, _: Context, stoken: String, userid: String, roomid: String) -> ErrorCode { //TODO: auth for all of these
		todo!()
	}

	async fn stop_typing(self, _: Context, stoken: String, userid: String, roomid: String) -> ErrorCode {
		todo!()
	}

	async fn keep_typing(self, _: Context, stoken: String, userid: String, roomid: String) -> ErrorCode {
		todo!()
	}

	async fn get_message_from_id(self, _: Context, stoken: String, id: i64) -> Result<Message, ErrorCode> {
		let is_admin = self.is_user_admin(&stoken).await;
		let result = sqlx::query(&format!("{}{}", FETCH_MESSAGE, "AND message.id = ?"))
			.bind(is_admin)
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

	async fn get_messages_since(self, _: Context, stoken: String, time: DateTime<Utc>) -> Result<Vec<Message>, ErrorCode> {
		let is_admin = self.is_user_admin(&stoken).await;
		let result = sqlx::query(&format!("{}{}", FETCH_MESSAGE, "AND message.timestamp >= ?"))
			.bind(is_admin)
			.bind(time)
			.fetch_all(&self.db_pool).await;
		
		match result {
			Ok(rows) => Ok(Message::from_rows(rows).unwrap()),
			Err(_) => Err(MalformedDBResponse)
		}
	}

	async fn get_all_direct_replies(self, _: Context, stoken: String, head: i64) -> Result<Vec<Message>, ErrorCode> {
		let is_admin = self.is_user_admin(&stoken).await;
		let result = sqlx::query(&format!("{}{}", FETCH_MESSAGE, "AND message.referencing_id = ?"))
			.bind(is_admin)
			.bind(head)
			.fetch_all(&self.db_pool).await;

		match result {
			Ok(rows) => Ok(Message::from_rows(rows).unwrap()),
			Err(_) => Err(MessageNotFound),
		}
	}

	async fn get_reply_chain(self, ctx: Context, stoken: String, head: Message, depth: u8) -> Result<ReplyChain, ErrorCode> {
		if depth > 8 {
			return Err(DepthTooLarge)
		}
		
		let direct_replies = self.clone().get_all_direct_replies(ctx, stoken.clone(), head.id).await?;
		let replies = if direct_replies.is_empty() || depth == 0 {
			None
		} else {
			let mut chains = Vec::new();
			
			for reply in direct_replies {
				chains.push(Box::pin(self.clone().get_reply_chain(ctx, stoken.clone(), reply, depth - 1)).await?);
			}
			
			Some(chains)
		};

		let chain = ReplyChain {
			message: head,
			replies,
		};

		Ok(chain)
	}

	async fn get_rooms(self, _: Context, stoken: String) -> Result<Vec<Room>, ErrorCode> {
		let is_admin = self.is_user_admin(&stoken).await;
		let result = query_as!(
			Room, "SELECT * FROM room WHERE admin_only_view = ? OR false", is_admin).fetch_all(&self.db_pool).await;

		match result {
			Ok(rooms) => Ok(rooms),
			Err(_) => Err(Error),
		}
	}

	async fn get_room(self, _: Context, stoken: String, roomid: String) -> Result<Room, ErrorCode> {
		let is_admin = self.is_user_admin(&stoken).await;
		let result = query_as!(
			Room, "SELECT * FROM room WHERE roomid = ? AND admin_only_view = ? OR false", is_admin, roomid).fetch_one(&self.db_pool).await;

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