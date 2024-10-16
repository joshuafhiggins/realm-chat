use chrono::{DateTime, Utc};
use sqlx::{FromRow, Row};
use sqlx::sqlite::SqliteRow;
use tarpc::serde::{Deserialize, Serialize};

use realm_shared::types::ErrorCode;
use crate::events::Event;
use crate::types::MessageData::*;

#[tarpc::service]
pub trait RealmChat {
	async fn test(name: String) -> String;
	
	async fn get_info() -> ServerInfo;
	async fn is_user_admin(stoken: String) -> bool;
	async fn is_user_owner(stoken: String) -> bool;
	async fn poll_events_since(index: u32) -> Vec<(u32, Event)>;
	async fn join_server(stoken: String, userid: String) -> Result<User, ErrorCode>;
	async fn leave_server(stoken: String, userid: String) -> Result<(), ErrorCode>;

	//NOTE: Any user authorized as themselves
	async fn send_message(stoken: String, message: Message) -> Result<Message, ErrorCode>;
	async fn start_typing(stoken: String, userid: String, roomid: String) -> ErrorCode;
	async fn stop_typing(stoken: String, userid: String, roomid: String) -> ErrorCode;
	async fn keep_typing(stoken: String, userid: String, roomid: String) -> ErrorCode; //NOTE: If a keep alive hasn't been received in 5 seconds, stop typing

	//NOTE: Any user can call, if they are in the server
	async fn get_message(stoken: String, userid: String, id: i64) -> Result<Message, ErrorCode>;
	async fn get_messages_since(stoken: String, userid: String, id: i64) -> Result<Vec<Message>, ErrorCode>;
	async fn get_all_direct_replies(stoken: String, userid: String, head: i64) -> Result<Vec<Message>, ErrorCode>;
	async fn get_reply_chain(stoken: String, userid: String, head: Message, depth: u8) -> Result<ReplyChain, ErrorCode>;
	async fn get_rooms(stoken: String, userid: String) -> Result<Vec<Room>, ErrorCode>;
	async fn get_room(stoken: String, userid: String, roomid: String) -> Result<Room, ErrorCode>;
	async fn get_user(userid: String) -> Result<User, ErrorCode>;
	async fn get_users() -> Result<Vec<User>, ErrorCode>;
	async fn create_room(stoken: String, userid: String, room: Room) -> Result<Room, ErrorCode>;
	async fn delete_room(stoken: String, userid: String, roomid: String) -> Result<(), ErrorCode>;
	async fn promote_user(stoken: String, admin_userid: String, userid: String) -> Result<(), ErrorCode>;
	async fn demote_user(stoken: String, owner_userid: String, userid: String) -> Result<(), ErrorCode>;
	async fn kick_user(stoken: String, admin_userid: String, userid: String) -> Result<(), ErrorCode>;
	async fn ban_user(stoken: String, admin_userid: String, userid: String) -> Result<(), ErrorCode>;
	async fn pardon_user(stoken: String, admin_userid: String, userid: String) -> Result<(), ErrorCode>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerInfo {
	pub server_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Message {
	pub id: i64,
	pub timestamp: DateTime<Utc>,
	pub user: User,
	pub room: Room,
	#[sqlx(flatten)]
	pub data: MessageData,
}

pub trait FromRows<R: Row>: Sized {
	fn from_rows(rows: Vec<R>) -> Result<Vec<Self>, sqlx::Error>;
}

impl FromRows<SqliteRow> for Message {
	fn from_rows(rows: Vec<SqliteRow>) -> sqlx::Result<Vec<Self>> {
		let mut messages = Vec::new();

		for row in rows {
			messages.push(Message::from_row(&row)?);
		}

		Ok(messages)
	}
}

impl FromRow<'_, SqliteRow> for Message {
	fn from_row(row: &SqliteRow) -> sqlx::Result<Self> {
		Ok(Self {
			id: row.try_get("id")?,
			timestamp: row.try_get("timestamp")?,
			user: User {
				id: row.try_get("user_id")?,
				userid: row.try_get("user_userid")?,
				name: row.try_get("user_name")?,
				owner: row.try_get("user_owner")?,
				admin: row.try_get("user_admin")?,
			},
			room: Room {
				id: row.try_get("room_id")?,
				roomid: row.try_get("room_roomid")?,
				admin_only_send: row.try_get("room_admin_only_send")?,
				admin_only_view: row.try_get("room_admin_only_view")?,
			},
			data: match row.try_get("msg_type")? {
				"text" => Text(row.try_get("msg_text")?),
				"attachment" => Attachment(Attachment {
					
				}),
				"reply" => Reply(Reply {
					referencing_id: row.try_get("referencing_id")?,
					text: row.try_get("msg_text")?,
				}),
				"edit" => Edit(Edit {
					referencing_id: row.try_get("referencing_id")?,
					text: row.try_get("msg_text")?,
				}),
				"reaction" => Reaction(Reaction {
					referencing_id: row.try_get("referencing_id")?,
					emoji: row.try_get("emoji")?,
				}),
				"redaction" => Redaction(Redaction {
					referencing_id: row.try_get("referencing_id")?,
				}),
				_ => { panic!() }
			},
		})
	}
}

//TODO: Maybe have multipart messages
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MessageData {
	Text(String),
	Attachment(Attachment),
	Reply(Reply),
	Edit(Edit), //NOTE: Have to be the owner of the referencing_guid
	Reaction(Reaction),
	Redaction(Redaction), //NOTE: Have to be the owner of the referencing_guid
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Attachment {
	//TODO
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Reply {
	pub referencing_id: i64,
	pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Edit {
	pub referencing_id: i64,
	pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Reaction {
	pub referencing_id: i64,
	pub emoji: String
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Redaction {
	pub referencing_id: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, PartialEq)]
pub struct User {
	pub id: i64,
	pub userid: String,
	pub name: String,
	pub owner: bool,
	pub admin: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, PartialEq)]
pub struct Room {
	pub id: i64,
	pub roomid: String,
	pub admin_only_send: bool,
	pub admin_only_view: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplyChain {
	pub message: Message,
	pub replies: Option<Vec<ReplyChain>>,
}

