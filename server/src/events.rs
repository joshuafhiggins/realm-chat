use crate::types::{Message, Room, User};

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum Event {
	// UserJoined(User),
	// UserLeft(User),
	None,
	NewMessage(Message),
	NewRoom(Room),
	DeleteRoom(String),
	// KickedUser(KickedUser),
	// BannedUser(BannedUser),
	// PromotedUser(PromotedUser),
	// DemotedUser(DemotedUser),
}