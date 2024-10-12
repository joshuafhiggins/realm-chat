#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct ClientUser {
	pub id: i64,
	pub server_address: String,
	pub username: String,
	pub email: String,
	//pub avatar: String,
	pub servers: Vec<String>,
	pub token: String,
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct ClientServer {
	pub server_id: String,
	pub domain: String,
	pub port: u16,
	pub is_admin: bool,
	pub is_owner: bool,
}