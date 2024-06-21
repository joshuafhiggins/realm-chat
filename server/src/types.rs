#[tarpc::service]
pub trait RealmChat {
	async fn test(name: String) -> String;
}

#[derive(Debug, Clone)]
pub struct Message {
	pub guid: String,
	pub text: Option<String>,
	pub attachments: Option<Vec<Attachment>>,
	pub reply_to_guid: Option<String>,
	pub reaction_emoji: Option<String>,
	pub redact: bool,
}

#[derive(Debug, Clone)]
pub struct Attachment {
	pub guid: String,
	//TODO
}