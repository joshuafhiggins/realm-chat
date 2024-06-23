use std::net::SocketAddr;
use futures::future;
use futures::future::Ready;
use tarpc::context::Context;
use crate::types::{Message, RealmChat, Room, SuccessCode, User};

#[derive(Clone)]
pub struct RealmChatServer(pub SocketAddr);

impl RealmChat for RealmChatServer {
	fn test(self, context: Context,  name: String) -> Ready<String> {
		future::ready(format!("Hello, {name}!"))
	}

	fn send_message(self, context: Context, message: Message) -> Ready<SuccessCode> {
		todo!()
	}

	async fn get_message_from_guid(self, context: Context, guid: String) -> Message {
		todo!()
	}

	async fn get_rooms(self, context: Context) -> Vec<Room> {
		todo!()
	}

	async fn get_room(self, context: Context, roomid: String) -> Room {
		todo!()
	}

	async fn get_user(self, context: Context, userid: String) -> User {
		todo!()
	}

	async fn get_joined_users(self, context: Context) -> Vec<User> {
		todo!()
	}

	async fn get_online_users(self, context: Context) -> Vec<User> {
		todo!()
	}
}