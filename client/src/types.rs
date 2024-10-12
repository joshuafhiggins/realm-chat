#[derive(serde::Deserialize, serde::Serialize)]
pub struct ClientUser {
	pub id: i64,
	pub username: String,
	pub email: String,
	pub avatar: String,
	pub servers: Vec<String>,
	pub token: String,
}