use chrono::{DateTime, TimeZone, Utc};
use tarpc::serde::{Deserialize, Serialize};

#[tarpc::service]
pub trait RealmChat {
	async fn test(name: String) -> String;
	
	//TODO: Any user authorized as themselves
	async fn send_message(message: Message) -> Result<Message, ErrorCode>;
	async fn start_typing() -> ErrorCode;
	async fn stop_typing() -> ErrorCode;
	async fn keep_typing() -> ErrorCode; //NOTE: If a keep alive hasn't been received in 5 seconds, stop typing

	//NOTE: Any user can call, if they are in the server
	async fn get_message_from_id(id: u32) -> Result<Message, ErrorCode>;
	async fn get_messages_since(time: DateTime<Utc>) -> Result<Vec<Message>, ErrorCode>;
	async fn get_rooms() -> Result<Vec<Room>, ErrorCode>;
	async fn get_room(roomid: String) -> Result<Room, ErrorCode>;
	async fn get_user(userid: String) -> Result<User, ErrorCode>;
	async fn get_users(get_only_online: bool) -> Result<Vec<User>, ErrorCode>;
	
	//TODO: Admin access only!
	// async fn create_room() -> Result<Room, ErrorCode>;
	// delete room
	// delete any message
	// kick user
	// ban user
	// unban user
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ErrorCode {
	None,
	Error,
	Unauthorized,
	NotFound,
	FailedToUnwrapDB,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
	pub id: u32,
	pub timestamp: DateTime<Utc>, //TODO: Does the database already have timestamps for us?
	pub user: User,
	pub room: Room,
	pub data: MessageData,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
	pub id: u32,
	pub userid: String,
	pub name: String,
	pub online: bool,
	pub admin: bool,
	//TODO: auth stuff needed, should be Option
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Room {
	pub id: u32,
	pub roomid: String,
	pub name: String,
	pub admin_only_send: bool,
	pub admin_only_view: bool,
}

