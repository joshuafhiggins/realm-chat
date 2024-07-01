use serde::{Deserialize, Serialize};

#[tarpc::service]
pub trait RealmAuth {
    async fn test(name: String) -> String;
    async fn server_token_validation(server_token: String, username: String, server_id: String, domain: String, tarpc_port: u16) -> bool;
    async fn create_account(username: String, email: String, avatar: String) -> Result<String, ErrorCode>;
    async fn create_login_flow(username: String) -> ErrorCode;
    async fn create_token_from_login(username: String, login_code: u16) -> Result<String, ErrorCode>;
    
    //NOTE: Need to be the user
    async fn change_email_flow(username: String, token: String) -> ErrorCode;
    async fn resolve_email_flow(username: String, token: String, login_code: u16, new_email: String) -> ErrorCode;
    async fn change_username(username: String, token: String, new_username: String) -> ErrorCode;
    async fn change_avatar(username: String, token: String, avatar: String) -> ErrorCode;
    async fn get_all_data(username: String, token: String) -> Result<AuthUser, ErrorCode>;
    
    //NOTE: Anyone can call
    async fn get_avatar_for_user(username: String) -> Result<String, ErrorCode>;
    //TODO:
    // Create account
    // Change email
    // Change username
    // Change/Upload/Delete avatar
    // OAuth login, check against email, store token, take avatar
    //      Google, Apple, GitHub, Discord
    // Get avatar
    // Get all userdata if you are the user
    // Server token validation
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ErrorCode {
    None,
    Error,
    EmailTaken,
    UsernameTaken,
    InvalidLoginCode,
    InvalidImage,
    InvalidUsername,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthUser {
    pub id: u32,
    pub username: String,
    pub email: String,
    pub avatar: String,
    pub login_code: Option<u16>,
    pub tokens: Option<Vec<String>>,
    pub google_oauth: Option<String>,
    pub apple_oauth: Option<String>,
    pub github_oauth: Option<String>,
    pub discord_oauth: Option<String>,
}