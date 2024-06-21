#[tarpc::service]
pub trait RealmChat {
	async fn test(name: String) -> String;
}

#[derive(Debug)]
pub struct Message {
	guid: String,
	text: Option<String>,
	attachments: Option<Vec<Attachment>>,
	reply_to_guid: Option<String>,
	reaction_emoji: Option<String>,
	redact: bool,
}

#[derive(Debug)]
pub struct Attachment {
	guid: String,
	//TODO
}