use std::net::SocketAddr;
use futures::future;
use futures::future::Ready;
use tarpc::context::Context;
use crate::types::{Message, RealmChat, Room, ErrorCode, User};

#[derive(Clone)]
pub struct RealmChatServer(pub SocketAddr);

impl RealmChat for RealmChatServer {
	fn test(self, context: Context,  name: String) -> Ready<String> {
		future::ready(format!("Hello, {name}!"))
	}
}