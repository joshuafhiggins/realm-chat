use serde::{Deserialize, Serialize};

#[tarpc::service]
pub trait RealmAuth {
    async fn test(name: String) -> String;
    async fn server_token_validation(server_token: String, username: String, server_id: String, domain: String, tarpc_port: u16) -> bool;
    async fn create_account_flow(username: String, email: String) -> Result<(), ErrorCode>; //NOTE: Still require sign in flow
    async fn create_login_flow(username: Option<String>, email: Option<String>) -> Result<(), ErrorCode>;
    async fn finish_login_flow(username: String, login_code: u16) -> Result<String, ErrorCode>;
    
    //NOTE: Need to be the user
    async fn change_email_flow(username: String, new_email: String, token: String) -> Result<(), ErrorCode>;
    async fn finish_change_email_flow(username: String, new_email: String, token: String, login_code: u16) -> Result<(), ErrorCode>;
    async fn change_username(username: String, token: String, new_username: String) -> Result<(), ErrorCode>;
    async fn change_avatar(username: String, token: String, new_avatar: String) -> Result<(), ErrorCode>;
    async fn get_all_data(username: String, token: String) -> Result<AuthUser, ErrorCode>;
    async fn sign_out(username: String, token: String) -> Result<(), ErrorCode>;
    
    //NOTE: Anyone can call
    async fn get_avatar_for_user(username: String) -> Result<String, ErrorCode>;
    // TODO: OAuth login, check against email, store token, take avatar: Google, Apple, GitHub, Discord
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ErrorCode {
    Error,
    Unauthorized,
    EmailTaken,
    UsernameTaken,
    InvalidLoginCode,
    InvalidImage,
    InvalidUsername,
    InvalidEmail,
    InvalidToken,
    UnableToConnectToMail,
    UnableToSendMail,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthUser {
    pub id: u32,
    pub username: String,
    pub email: String,
    pub avatar: String,
    pub login_code: Option<u16>,
    pub bigtoken: Option<String>,
    pub google_oauth: Option<String>,
    pub apple_oauth: Option<String>,
    pub github_oauth: Option<String>,
    pub discord_oauth: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthEmail {
    pub server_address: String,
    pub server_port: u16,
    pub auth_name: String,
    pub auth_from_address: String,
    pub auth_username: String,
    pub auth_password: String,
}