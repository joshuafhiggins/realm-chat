use chrono::{DateTime, Utc};
use sqlx::{FromRow, Row};
use sqlx::sqlite::SqliteRow;
use tarpc::serde::{Deserialize, Serialize};

use realm_shared::types::ErrorCode;

use crate::types::MessageData::*;

#[tarpc::service]
pub trait RealmChat {
	async fn test(name: String) -> String;

	//TODO: Any user authorized as themselves
	async fn send_message(auth_token: String, message: Message) -> Result<Message, ErrorCode>;
	async fn start_typing(auth_token: String) -> ErrorCode;
	async fn stop_typing(auth_token: String) -> ErrorCode;
	async fn keep_typing(auth_token: String) -> ErrorCode; //NOTE: If a keep alive hasn't been received in 5 seconds, stop typing

	//NOTE: Any user can call, if they are in the server
	async fn get_message_from_id(auth_token: String, id: u32) -> Result<Message, ErrorCode>;
	async fn get_messages_since(auth_token: String, time: DateTime<Utc>) -> Result<Vec<Message>, ErrorCode>;
	async fn get_rooms(auth_token: String) -> Result<Vec<Room>, ErrorCode>;
	async fn get_room(auth_token: String, roomid: String) -> Result<Room, ErrorCode>;
	async fn get_user(userid: String) -> Result<User, ErrorCode>;
	async fn get_users() -> Result<Vec<User>, ErrorCode>;
	async fn get_online_users() -> Result<Vec<User>, ErrorCode>;

	//TODO: Admin access only!
	// async fn create_room() -> Result<Room, ErrorCode>;
	// delete room
	// delete any message
	// kick user
	// ban user
	// unban user
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

impl FromRow<'_, SqliteRow> for Message {
	fn from_row(row: &SqliteRow) -> sqlx::Result<Self> {
		Ok(Self {
			id: row.try_get("id")?,
			timestamp: row.try_get("timestamp")?,
			user: User {
				id: row.try_get("user_id")?,
				userid: row.try_get("user_userid")?,
				name: row.try_get("user_name")?,
				online: row.try_get("user_online")?,
				admin: row.try_get("user_admin")?,
			},
			room: Room {
				id: row.try_get("room_id")?,
				roomid: row.try_get("room_roomid")?,
				name: row.try_get("room_name")?,
				admin_only_send: row.try_get("room_admin_only_send")?,
				admin_only_view: row.try_get("room_admin_only_view")?,
			},
			data: match row.try_get("msg_type")? {
				"text" => Text(row.try_get("msg_text")?),
				"attachment" => todo!(),
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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageData {
	Text(String),
	Attachment(Attachment),
	Reply(Reply),
	Edit(Edit), //NOTE: Have to be the owner of the referencing_guid
	Reaction(Reaction),
	Redaction(Redaction), //NOTE: Have to be the owner of the referencing_guid
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attachment {
	//TODO
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reply {
	pub referencing_id: u32,
	pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Edit {
	pub referencing_id: u32,
	pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reaction {
	pub referencing_id: u32,
	pub emoji: String
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Redaction {
	pub referencing_id: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
	pub id: i64,
	pub userid: String,
	pub name: String,
	pub online: bool,
	pub admin: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Room {
	pub id: i64,
	pub roomid: String,
	pub name: String,
	pub admin_only_send: bool,
	pub admin_only_view: bool,
}

