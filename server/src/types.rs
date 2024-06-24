use tarpc::serde::{Deserialize, Serialize};

#[tarpc::service]
pub trait RealmChat {
	async fn test(name: String) -> String;
	async fn send_message(message: Message) -> ErrorCode;

	async fn get_message_from_guid(guid: String) -> Result<Message, ErrorCode>;
	async fn get_rooms() -> Result<Vec<Room>, ErrorCode>;
	async fn get_room(roomid: String) -> Result<Room, ErrorCode>;
	async fn get_user(userid: String) -> Result<User, ErrorCode>;
	async fn get_joined_users() -> Result<Vec<User>, ErrorCode>;
	async fn get_online_users() -> Result<Vec<User>, ErrorCode>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ErrorCode {
	None = 0,
	Error,
	Unauthorized,
	NotFound,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
	pub guid: String,
	pub timestamp: u64,
	pub user: User,
	pub room: Room,
	pub text: Option<String>,
	pub attachments: Option<Vec<Attachment>>,
	pub reply_to_guid: Option<String>,
	pub reaction_emoji: Option<String>,
	pub redact: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
	pub userid: String,
	pub name: String,
	//TODO
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Room {
	pub roomid: String,
	//TODO
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attachment {
	pub guid: String,
	//TODO
}