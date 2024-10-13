#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct CUser {
	pub id: i64,
	pub auth_address: String,
	pub username: String,
	pub email: String,
	//pub avatar: String,
	pub server_addresses: Vec<String>,
	pub token: String,
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct CServer {
	pub server_id: String,
	pub domain: String,
	pub port: u16,
	pub is_admin: bool,
	pub is_owner: bool,
}