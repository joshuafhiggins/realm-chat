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
	async fn get_message_from_guid(guid: String) -> Result<Message, ErrorCode>;
	async fn get_messages_since(time: u64) -> Result<Vec<Message>, ErrorCode>;
	async fn get_rooms() -> Result<Vec<Room>, ErrorCode>;
	async fn get_room(roomid: String) -> Result<Room, ErrorCode>;
	async fn get_user(userid: String) -> Result<User, ErrorCode>;
	async fn get_joined_users() -> Result<Vec<User>, ErrorCode>;
	async fn get_online_users() -> Result<Vec<User>, ErrorCode>;
	
	//TODO: Admin access only!
	// async fn create_room() -> Result<Room, ErrorCode>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ErrorCode {
	None,
	Error,
	Unauthorized,
	NotFound,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
	pub id: u32,
	pub timestamp: u64, //TODO: Change to a real time for SQL
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
	Redaction(Redaction),
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
	//TODO
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Room {
	pub id: u32,
	pub roomid: String,
	pub name: String,
	//TODO
}

